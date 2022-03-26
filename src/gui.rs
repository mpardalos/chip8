use std::collections::HashSet;
use std::{sync::Mutex, time::Instant};

use sdl2::keyboard::Scancode;
use sdl2::{
    event::Event,
    pixels::Color,
    rect::Rect,
    render::{Canvas, TextureQuery, WindowCanvas},
    ttf::Font,
    video::Window,
};

use crate::cpu::{CHIP8, CHIP8IO};
use crate::instruction::Instruction;
use crate::{
    cpu::{DISPLAY_COLS, DISPLAY_ROWS},
    rate_limit,
};

const WINDOW_NAME: &str = "CHIP8";
const DISPLAY_WIDTH: u32 = 960;
const DISPLAY_HEIGHT: u32 = 540;
const PIXEL_WIDTH: u32 = DISPLAY_WIDTH / DISPLAY_COLS as u32;
const PIXEL_HEIGHT: u32 = DISPLAY_HEIGHT / DISPLAY_ROWS as u32;

const WINDOW_WIDTH: u32 = DISPLAY_WIDTH + 300;
const WINDOW_HEIGHT: u32 = DISPLAY_HEIGHT + 200;

pub fn run_gui(fps: u64, cpu: &Mutex<CHIP8>, io: &Mutex<CHIP8IO>) -> Result<(), String> {
    // Load a font
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let mut font = ttf_context
        .load_font("/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf", 18)
        .unwrap();
    font.set_style(sdl2::ttf::FontStyle::BOLD);

    let sdl_context = sdl2::init().map_err(|e| e.to_string())?;
    let mut canvas: Canvas<Window> = sdl_context
        .video()
        .map_err(|e| e.to_string())?
        .window(WINDOW_NAME, WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump().map_err(|e| e.to_string())?;

    let mut checked_keys: HashSet<u8> = HashSet::new();
    let mut checked_registers: HashSet<u8> = HashSet::new();
    let mut ticker = Instant::now();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        // Take input
        let keyboard_state = event_pump.keyboard_state();
        {
            let keystate = &mut io.lock().unwrap().keystate;
            keystate[0x0] = keyboard_state.is_scancode_pressed(Scancode::Num1);
            keystate[0x1] = keyboard_state.is_scancode_pressed(Scancode::Num2);
            keystate[0x2] = keyboard_state.is_scancode_pressed(Scancode::Num3);
            keystate[0x3] = keyboard_state.is_scancode_pressed(Scancode::Num4);
            keystate[0x4] = keyboard_state.is_scancode_pressed(Scancode::Q);
            keystate[0x5] = keyboard_state.is_scancode_pressed(Scancode::W);
            keystate[0x6] = keyboard_state.is_scancode_pressed(Scancode::E);
            keystate[0x7] = keyboard_state.is_scancode_pressed(Scancode::R);
            keystate[0x8] = keyboard_state.is_scancode_pressed(Scancode::A);
            keystate[0x9] = keyboard_state.is_scancode_pressed(Scancode::S);
            keystate[0xA] = keyboard_state.is_scancode_pressed(Scancode::D);
            keystate[0xB] = keyboard_state.is_scancode_pressed(Scancode::F);
            keystate[0xC] = keyboard_state.is_scancode_pressed(Scancode::Z);
            keystate[0xD] = keyboard_state.is_scancode_pressed(Scancode::X);
            keystate[0xE] = keyboard_state.is_scancode_pressed(Scancode::C);
            keystate[0xF] = keyboard_state.is_scancode_pressed(Scancode::V);
        }

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();

        // Draw register state
        {
            let mut x: i32 = DISPLAY_WIDTH as i32 + 10;
            let mut y: i32 = 0;
            let register_state = cpu.lock().unwrap().reg;
            {
                canvas.set_draw_color(Color::YELLOW);
                canvas.draw_rect(Rect::new(
                    x,
                    y,
                    font.size_of_char('O').unwrap().0 as u32 * 10,
                    font.height() as u32 * register_state.len() as u32,
                ))?;

                for (reg, val) in register_state.iter().enumerate() {
                    show_text(
                        &mut canvas,
                        &font,
                        TextBackground::Transparent,
                        x,
                        y,
                        &format!("v{:X} | {:#x}", reg, val),
                    )?;
                    y += font.height();
                }
            }

            // Draw keypad
            {
                let start_x = x;
                y += 5;
                const SIZE: u32 = 30;

                canvas.set_draw_color(Color::RED);
                let keystate = io.lock().unwrap().keystate;
                for (key, &pressed) in keystate.iter().enumerate() {
                    if key % 4 == 0 {
                        x = start_x;
                        y += SIZE as i32;
                    }

                    // if pressed {
                    //     canvas.fill_rect(Rect::new(x, y, SIZE, SIZE))?;
                    // } else {
                    //     canvas.draw_rect(Rect::new(x, y, SIZE, SIZE))?;
                    // }

                    show_text(
                        &mut canvas,
                        &font,
                        if pressed {
                            TextBackground::Solid(Color::RED)
                        } else {
                            TextBackground::Transparent
                        },
                        x,
                        y,
                        &format!("{:X}", key),
                    )?;

                    x += SIZE as i32;
                }
            }

            // Draw waiting for key
            {
                x = 10;
                y = DISPLAY_HEIGHT as i32 + 10;
                if let Ok(current_instr) = cpu.lock().unwrap().current_instruction() {
                    let text = match current_instr {
                        Instruction::SKPR(r) => {
                            let key = register_state[r as usize];
                            checked_registers.insert(r);
                            checked_keys.insert(key);
                            format!("Checking {:X}", key)
                        }
                        Instruction::SKUP(r) => {
                            let key = register_state[r as usize];
                            checked_registers.insert(r);
                            checked_keys.insert(key);
                            format!("Checking {:X}", key)
                        }
                        Instruction::KEYD(_) => format!("Waiting for a key"),
                        _ => format!(" "),
                    };
                    show_text(&mut canvas, &font, TextBackground::Transparent, x, y, &text)?;
                    y += font.height();
                    show_text(
                        &mut canvas,
                        &font,
                        TextBackground::Transparent,
                        x,
                        y,
                        &format!("Checked keys: {:?}", checked_keys),
                    )?;
                    y += font.height();
                    show_text(
                        &mut canvas,
                        &font,
                        TextBackground::Transparent,
                        x,
                        y,
                        &format!("Checked registers: {:?}", checked_registers),
                    )?;
                }
            }
        }

        // Draw display
        {
            let mut y: u32 = 0;
            for row in io.lock().unwrap().display {
                let mut x: u32 = 0;
                for pixel in row {
                    canvas.set_draw_color(if pixel { Color::BLUE } else { Color::BLACK });
                    canvas.fill_rect(Rect::new(x as i32, y as i32, PIXEL_WIDTH, PIXEL_HEIGHT))?;
                    x += PIXEL_WIDTH;
                }
                y += PIXEL_HEIGHT;
            }

            // Frame timing
            let (_, frame_time) = rate_limit(fps, &mut ticker);
            show_text(
                &mut canvas,
                &font,
                TextBackground::Solid(Color::BLACK),
                0,
                0,
                &format!("{:.0}   ", 1. / frame_time.as_secs_f32()),
            )?;

            canvas.set_draw_color(Color::YELLOW);
            canvas.draw_rect(Rect::new(0, 0, DISPLAY_WIDTH, DISPLAY_HEIGHT))?;
        }
        canvas.present();
    }

    Ok(())
}

#[allow(dead_code)]
enum TextBackground {
    Solid(Color),
    Transparent,
}

fn show_text(
    canvas: &mut WindowCanvas,
    font: &Font,
    background: TextBackground,
    x: i32,
    y: i32,
    text: &str,
) -> Result<Rect, String> {
    let texture_creator = canvas.texture_creator();
    let surface = match background {
        TextBackground::Solid(bg) => font.render(text).shaded(Color::WHITE, bg),
        TextBackground::Transparent => font.render(text).blended(Color::WHITE),
    }
    .map_err(|e| e.to_string())?;
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;
    let TextureQuery { width, height, .. } = texture.query();
    let target_rect = Rect::new(x, y, width, height);
    canvas.copy(&texture, None, Some(target_rect))?;
    Ok(target_rect)
}
