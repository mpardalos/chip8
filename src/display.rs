use std::time::{Duration, Instant};

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::Rect,
    render::{Canvas, TextureQuery, WindowCanvas},
    ttf::Font,
    video::Window,
};

use crate::cpu::CHIP8;

const WINDOW_NAME: &str = "CHIP8";
const WINDOW_WIDTH: u32 = 960;
const WINDOW_HEIGHT: u32 = 540;
const WINDOW_FPS: u32 = 60;

pub fn run_window(mut cpu: CHIP8) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();
    let window = video_subsystem
        .window(WINDOW_NAME, WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas: Canvas<Window> = window.into_canvas().build().unwrap();

    // Load a font
    let mut font = ttf_context
        .load_font("/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf", 18)
        .unwrap();
    font.set_style(sdl2::ttf::FontStyle::BOLD);

    let mut target_fps = WINDOW_FPS;
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut frame_time_counter = Instant::now();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape | Keycode::Q),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => { /* reset */ }
                Event::KeyDown {
                    keycode: Some(Keycode::O),
                    ..
                } => target_fps = target_fps.saturating_sub(1),
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => target_fps = target_fps.saturating_add(1),
                _ => {}
            }
        }

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / target_fps));

        let now = Instant::now();
        let frametime = now - frame_time_counter;
        frame_time_counter = now;
        show_text(
            &mut canvas,
            &font,
            TextBackground::Solid(Color::BLACK),
            0,
            0,
            &format!("{:.0}", 1. / frametime.as_secs_f32()),
        )
        .unwrap();
        canvas.present();
    }
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