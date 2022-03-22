use std::{
    cmp::min,
    time::{Duration, Instant},
};

use sdl2::{
    event::Event,
    keyboard::{Keycode, Scancode},
    pixels::Color,
    rect::Rect,
    render::{Canvas, TextureQuery, WindowCanvas},
    ttf::Font,
    video::Window,
};

use crate::cpu::{StepResult, CHIP8, DISPLAY_COLS, DISPLAY_ROWS};

const WINDOW_NAME: &str = "CHIP8";
const WINDOW_WIDTH: u32 = 960;
const WINDOW_HEIGHT: u32 = 540;
const WINDOW_FPS: u32 = 60;
const TARGET_STEP_TIME: Duration = Duration::from_nanos(016666666);

pub fn run_window(mut cpu: CHIP8) -> Result<(), String> {
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

    // Load a font
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let mut font = ttf_context
        .load_font("/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf", 18)
        .unwrap();
    font.set_style(sdl2::ttf::FontStyle::BOLD);

    'running: loop {
        let tick_start = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        let keyboard_state = event_pump.keyboard_state();
        let keystate = [
            keyboard_state.is_scancode_pressed(Scancode::Num1),
            keyboard_state.is_scancode_pressed(Scancode::Num2),
            keyboard_state.is_scancode_pressed(Scancode::Num3),
            keyboard_state.is_scancode_pressed(Scancode::Num4),
            keyboard_state.is_scancode_pressed(Scancode::Q),
            keyboard_state.is_scancode_pressed(Scancode::W),
            keyboard_state.is_scancode_pressed(Scancode::E),
            keyboard_state.is_scancode_pressed(Scancode::R),
            keyboard_state.is_scancode_pressed(Scancode::A),
            keyboard_state.is_scancode_pressed(Scancode::S),
            keyboard_state.is_scancode_pressed(Scancode::D),
            keyboard_state.is_scancode_pressed(Scancode::F),
            keyboard_state.is_scancode_pressed(Scancode::Z),
            keyboard_state.is_scancode_pressed(Scancode::X),
            keyboard_state.is_scancode_pressed(Scancode::C),
            keyboard_state.is_scancode_pressed(Scancode::V),
        ];
        match cpu.step(&keystate)? {
            StepResult::End => break 'running,
            StepResult::Continue(false) => {}
            StepResult::Continue(true) => {
                // Draw display
                canvas.clear();
                let (win_width, win_height) = canvas.window().size();
                let pixel_width: u32 = win_width / DISPLAY_COLS as u32;
                let pixel_height: u32 = win_height / DISPLAY_ROWS as u32;
                let mut y: u32 = 0;
                for row in cpu.display {
                    let mut x: u32 = 0;
                    for pixel in row {
                        canvas.set_draw_color(if pixel { Color::BLUE } else { Color::BLACK });
                        canvas.fill_rect(Rect::new(
                            x as i32,
                            y as i32,
                            pixel_width,
                            pixel_height,
                        ))?;
                        x += pixel_width;
                    }
                    y += pixel_height;
                }
            }
        }

        // Frame timing
        let tick_end = Instant::now();
        let elapsed = tick_end - tick_start;
        std::thread::sleep(TARGET_STEP_TIME - min(TARGET_STEP_TIME, elapsed));
        let frame_time = Instant::now() - tick_start;
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
