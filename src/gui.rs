use std::sync::atomic::{self, AtomicU64};
use std::sync::{Arc, Mutex};

use eframe::egui::Slider;
use eframe::epaint::{Color32, Rect, Vec2};
use eframe::{egui, epi};

use crate::cpu::{Chip8, Chip8IO, StepResult, KEYPAD_TO_QWERTY};
use crate::cpu::{DISPLAY_COLS, DISPLAY_ROWS};

const WINDOW_NAME: &str = "CHIP8";
const DISPLAY_WIDTH: f32 = 960.;
const DISPLAY_HEIGHT: f32 = 540.;
const PIXEL_WIDTH: f32 = DISPLAY_WIDTH / DISPLAY_COLS as f32;
const PIXEL_HEIGHT: f32 = DISPLAY_HEIGHT / DISPLAY_ROWS as f32;

const WINDOW_WIDTH: f32 = DISPLAY_WIDTH + 300.;
const WINDOW_HEIGHT: f32 = DISPLAY_HEIGHT + 200.;

pub struct Chip8Gui {
    cpu: Arc<Mutex<Chip8>>,
    io: Arc<Mutex<Chip8IO>>,

    target_ips: Arc<AtomicU64>,
    dark_mode: bool,
}

impl Chip8Gui {
    pub fn new(
        cpu: Arc<Mutex<Chip8>>,
        io: Arc<Mutex<Chip8IO>>,
        target_ips: Arc<AtomicU64>,
        dark_mode: bool,
    ) -> Self {
        Self {
            cpu,
            io,
            target_ips,
            dark_mode,
        }
    }

    pub fn run(self) {
        eframe::run_native(
            Box::new(self),
            eframe::NativeOptions {
                initial_window_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
                ..eframe::NativeOptions::default()
            },
        );
    }

    fn chip8_display(&self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(DISPLAY_WIDTH, DISPLAY_HEIGHT),
            egui::Sense {
                click: false,
                drag: false,
                focusable: false,
            },
        );

        let (off_color, on_color) = if ui.style().visuals.dark_mode {
            (Color32::BLACK, Color32::WHITE)
        } else {
            (Color32::WHITE, Color32::BLACK)
        };

        let mut pos = rect.min;
        for row in self.io.lock().unwrap().display {
            pos.x = 0.;
            for pixel in row {
                ui.painter().rect(
                    Rect::from_min_size(pos, Vec2::new(PIXEL_WIDTH + 1., PIXEL_HEIGHT + 1.)),
                    0.,
                    if pixel { on_color } else { off_color },
                    (0., off_color),
                );
                pos.x += PIXEL_WIDTH;
            }
            pos.y += PIXEL_HEIGHT as f32;
        }

        response
    }

    fn draw_keypad(&self, ui: &mut egui::Ui) -> egui::Response {
        egui::Grid::new("chip8_keypad")
            .show(ui, |ui| {
                for (idx, &keypad_key) in KEYPAD_TO_QWERTY.keys().enumerate() {
                    let pressed = self.io.lock().unwrap().keystate[keypad_key as usize];
                    if idx % 4 == 0 && (idx != 0) {
                        ui.end_row();
                    }

                    ui.label(
                        egui::RichText::new(&format!("{:X}", keypad_key)).background_color(
                            if pressed {
                                Color32::RED
                            } else {
                                Color32::TRANSPARENT
                            },
                        ),
                    );
                }
            })
            .response
    }

    fn draw_registers(&self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            egui::Grid::new("chip8_keypad")
                .show(ui, |ui| {
                    let cpu = self.cpu.lock().unwrap();
                    for (reg, val) in cpu.reg.iter().enumerate() {
                        ui.label(format!("v{:X}", reg));
                        ui.label(format!("v{:#x}", val));
                        ui.end_row();
                    }
                    ui.label("Index");
                    ui.label(format!("v{:#x}", cpu.idx));
                    ui.end_row();
                })
                .response;
            let (pc, instr) = {
                let cpu = self.cpu.lock().unwrap();
                (cpu.pc, cpu.current_instruction())
            };
            ui.label(format!(
                "At [{:#x}]: {}",
                pc,
                match instr {
                    Ok(i) => format!("{}", i),
                    Err(_) => "???".to_string(),
                }
            ));
        })
        .response
    }

    fn run_controls(&mut self, ui: &mut egui::Ui) {
        if let Ok(mut cpu) = self.cpu.lock() {
            if ui.button("Reset").clicked() {
                cpu.reset();
            }
            ui.checkbox(&mut cpu.paused, "Pause");
            if cpu.paused {
                if ui.button("Step").clicked() {
                    cpu.paused = false;
                    let _ = cpu.step();
                    cpu.paused = true;
                }
                if ui.button("Step to display update").clicked() {
                    cpu.paused = false;
                    while cpu.step() != Ok(StepResult::Continue(true)) {}
                    cpu.paused = true;
                }
            }
        }
    }
}

impl epi::App for Chip8Gui {
    fn name(&self) -> &str {
        WINDOW_NAME
    }

    fn setup(
        &mut self,
        ctx: &egui::Context,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        ctx.set_style(egui::Style {
            visuals: if self.dark_mode {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            },
            override_font_id: Some(egui::FontId::proportional(22.)),
            ..egui::Style::default()
        })
    }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        {
            let chip8_keys = &mut self.io.lock().unwrap().keystate;
            let pressed_keys = &ctx.input().keys_down;
            for key in 0..chip8_keys.len() {
                chip8_keys[key] =
                    pressed_keys.contains(&key_for_char(KEYPAD_TO_QWERTY[&(key as u8)]).unwrap())
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.run_controls(ui);
                ui.add(
                    Slider::from_get_set(1.0..=3000.0, |set_val| {
                        if let Some(val) = set_val {
                            self.target_ips.store(val as u64, atomic::Ordering::Relaxed);
                        }
                        self.target_ips.load(atomic::Ordering::Relaxed) as f64
                    })
                    .text("Target IPS"),
                );
            });
            ui.separator();
            ui.horizontal(|ui| {
                self.chip8_display(ui);
                ui.vertical(|ui| {
                    self.draw_registers(ui);
                    self.draw_keypad(ui);
                });
            });
        });

        frame.request_repaint();
    }
}

fn key_for_char(value: char) -> Option<egui::Key> {
    match value {
        '1' => Some(egui::Key::Num1),
        '2' => Some(egui::Key::Num2),
        '3' => Some(egui::Key::Num3),
        '4' => Some(egui::Key::Num4),
        '5' => Some(egui::Key::Num5),
        '6' => Some(egui::Key::Num6),
        '7' => Some(egui::Key::Num7),
        '8' => Some(egui::Key::Num8),
        '9' => Some(egui::Key::Num9),
        '0' => Some(egui::Key::Num0),
        'q' | 'Q' => Some(egui::Key::Q),
        'w' | 'W' => Some(egui::Key::W),
        'e' | 'E' => Some(egui::Key::E),
        'r' | 'R' => Some(egui::Key::R),
        't' | 'T' => Some(egui::Key::T),
        'y' | 'Y' => Some(egui::Key::Y),
        'u' | 'U' => Some(egui::Key::U),
        'i' | 'I' => Some(egui::Key::I),
        'o' | 'O' => Some(egui::Key::O),
        'p' | 'P' => Some(egui::Key::P),
        'a' | 'A' => Some(egui::Key::A),
        's' | 'S' => Some(egui::Key::S),
        'd' | 'D' => Some(egui::Key::D),
        'f' | 'F' => Some(egui::Key::F),
        'g' | 'G' => Some(egui::Key::G),
        'h' | 'H' => Some(egui::Key::H),
        'j' | 'J' => Some(egui::Key::J),
        'k' | 'K' => Some(egui::Key::K),
        'l' | 'L' => Some(egui::Key::L),
        'z' | 'Z' => Some(egui::Key::Z),
        'x' | 'X' => Some(egui::Key::X),
        'c' | 'C' => Some(egui::Key::C),
        'v' | 'V' => Some(egui::Key::V),
        'b' | 'B' => Some(egui::Key::B),
        'n' | 'N' => Some(egui::Key::N),
        'm' | 'M' => Some(egui::Key::M),
        _ => None,
    }
}
