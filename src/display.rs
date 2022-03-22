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

use crate::cpu::CHIP8IO;
use crate::{
    cpu::{DISPLAY_COLS, DISPLAY_ROWS},
    rate_limit,
};

const WINDOW_NAME: &str = "CHIP8";
const DISPLAY_WIDTH: u32 = 960;
const DISPLAY_HEIGHT: u32 = 540;

pub fn run_gui(fps: u64, io: &Mutex<CHIP8IO>) -> Result<(), String> {
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
        .window(
            WINDOW_NAME,
            DISPLAY_WIDTH,
            DISPLAY_HEIGHT + font.height() as u32,
        )
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump().map_err(|e| e.to_string())?;

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
            keystate[0x1] = keyboard_state.is_scancode_pressed(Scancode::Num1);
            keystate[0x2] = keyboard_state.is_scancode_pressed(Scancode::Num2);
            keystate[0x3] = keyboard_state.is_scancode_pressed(Scancode::Num3);
            keystate[0xC] = keyboard_state.is_scancode_pressed(Scancode::Num4);
            keystate[0x4] = keyboard_state.is_scancode_pressed(Scancode::Q);
            keystate[0x5] = keyboard_state.is_scancode_pressed(Scancode::W);
            keystate[0x6] = keyboard_state.is_scancode_pressed(Scancode::E);
            keystate[0xD] = keyboard_state.is_scancode_pressed(Scancode::R);
            keystate[0x7] = keyboard_state.is_scancode_pressed(Scancode::A);
            keystate[0x8] = keyboard_state.is_scancode_pressed(Scancode::S);
            keystate[0x9] = keyboard_state.is_scancode_pressed(Scancode::D);
            keystate[0xE] = keyboard_state.is_scancode_pressed(Scancode::F);
            keystate[0xA] = keyboard_state.is_scancode_pressed(Scancode::Z);
            keystate[0x0] = keyboard_state.is_scancode_pressed(Scancode::X);
            keystate[0xB] = keyboard_state.is_scancode_pressed(Scancode::C);
            keystate[0xF] = keyboard_state.is_scancode_pressed(Scancode::V);
        }

        // Draw display
        canvas.clear();
        let (win_width, win_height) = canvas.window().size();
        let pixel_width: u32 = win_width / DISPLAY_COLS as u32;
        let pixel_height: u32 = win_height / DISPLAY_ROWS as u32;
        let mut y: u32 = 0;
        for row in io.lock().unwrap().display {
            let mut x: u32 = 0;
            for pixel in row {
                canvas.set_draw_color(if pixel { Color::BLUE } else { Color::BLACK });
                canvas.fill_rect(Rect::new(x as i32, y as i32, pixel_width, pixel_height))?;
                x += pixel_width;
            }
            y += pixel_height;
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
