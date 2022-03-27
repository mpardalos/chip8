use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use eframe::epaint::{Color32, Rect, Vec2};
use eframe::{egui, epi};

use crate::cpu::{CHIP8, CHIP8IO};
use crate::cpu::{DISPLAY_COLS, DISPLAY_ROWS};
use crate::instruction::Instruction;

const WINDOW_NAME: &str = "CHIP8";
const DISPLAY_WIDTH: f32 = 960.;
const DISPLAY_HEIGHT: f32 = 540.;
const PIXEL_WIDTH: f32 = DISPLAY_WIDTH / DISPLAY_COLS as f32;
const PIXEL_HEIGHT: f32 = DISPLAY_HEIGHT / DISPLAY_ROWS as f32;

const WINDOW_WIDTH: f32 = DISPLAY_WIDTH + 300.;
const WINDOW_HEIGHT: f32 = DISPLAY_HEIGHT + 200.;

pub struct Chip8Gui {
    cpu: Arc<Mutex<CHIP8>>,
    io: Arc<Mutex<CHIP8IO>>,

    checked_keys: HashSet<u8>,
    checked_registers: HashSet<u8>,
}

impl Chip8Gui {
    pub fn new(cpu: Arc<Mutex<CHIP8>>, io: Arc<Mutex<CHIP8IO>>) -> Self {
        Self {
            cpu,
            io,
            checked_keys: HashSet::new(),
            checked_registers: HashSet::new(),
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

        let mut pos = rect.min;
        for row in self.io.lock().unwrap().display {
            pos.x = 0.;
            for pixel in row {
                ui.painter().rect(
                    Rect::from_min_size(pos, Vec2::new(PIXEL_WIDTH + 1., PIXEL_HEIGHT + 1.)),
                    0.,
                    if pixel { Color32::BLUE } else { Color32::BLACK },
                    (0., Color32::BLUE),
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
                for (key, &pressed) in self.io.lock().unwrap().keystate.iter().enumerate() {
                    if key % 4 == 0 && (key != 0) {
                        ui.end_row();
                    }

                    ui.label(egui::RichText::new(&format!("{:X}", key)).background_color(
                        if pressed {
                            Color32::RED
                        } else {
                            Color32::TRANSPARENT
                        },
                    ));
                }
            })
            .response
    }

    fn draw_registers(&self, ui: &mut egui::Ui) -> egui::Response {
        egui::Grid::new("chip8_keypad")
            .show(ui, |ui| {
                for (reg, val) in self.cpu.lock().unwrap().reg.iter().enumerate() {
                    ui.label(format!("v{:X}", reg));
                    ui.label(format!("v{:#x}", val));
                    ui.end_row();
                }
            })
            .response
    }

    fn draw_input_checking_state(&mut self, ui: &mut egui::Ui) {
        let register_state = self.cpu.lock().unwrap().reg;
        if let Ok(current_instr) = self.cpu.lock().unwrap().current_instruction() {
            let text = match current_instr {
                Instruction::SKPR(r) => {
                    let key = register_state[r as usize];
                    self.checked_registers.insert(r);
                    self.checked_keys.insert(key);
                    format!("Checking {:X}", key)
                }
                Instruction::SKUP(r) => {
                    let key = register_state[r as usize];
                    self.checked_registers.insert(r);
                    self.checked_keys.insert(key);
                    format!("Checking {:X}", key)
                }
                Instruction::KEYD(_) => format!("Waiting for a key"),
                _ => format!(" "),
            };
            ui.vertical(|ui| {
                ui.label(text);
                ui.label(&format!("Checked keys: {:?}", self.checked_keys));
                ui.label(&format!("Checked registers: {:?}", self.checked_registers))
            });
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
        ctx.set_visuals(egui::Visuals::dark())
    }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        // Take input
        {
            let chip8_keys = &mut self.io.lock().unwrap().keystate;
            let pressed_keys = &ctx.input().keys_down;

            chip8_keys[0x0] = pressed_keys.contains(&egui::Key::Num1);
            chip8_keys[0x1] = pressed_keys.contains(&egui::Key::Num2);
            chip8_keys[0x2] = pressed_keys.contains(&egui::Key::Num3);
            chip8_keys[0x3] = pressed_keys.contains(&egui::Key::Num4);
            chip8_keys[0x4] = pressed_keys.contains(&egui::Key::Q);
            chip8_keys[0x5] = pressed_keys.contains(&egui::Key::W);
            chip8_keys[0x6] = pressed_keys.contains(&egui::Key::E);
            chip8_keys[0x7] = pressed_keys.contains(&egui::Key::R);
            chip8_keys[0x8] = pressed_keys.contains(&egui::Key::A);
            chip8_keys[0x9] = pressed_keys.contains(&egui::Key::S);
            chip8_keys[0xA] = pressed_keys.contains(&egui::Key::D);
            chip8_keys[0xB] = pressed_keys.contains(&egui::Key::F);
            chip8_keys[0xC] = pressed_keys.contains(&egui::Key::Z);
            chip8_keys[0xD] = pressed_keys.contains(&egui::Key::X);
            chip8_keys[0xE] = pressed_keys.contains(&egui::Key::C);
            chip8_keys[0xF] = pressed_keys.contains(&egui::Key::V);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    self.chip8_display(ui);
                    self.draw_input_checking_state(ui);
                });
                ui.vertical(|ui| {
                    self.draw_registers(ui);
                    self.draw_keypad(ui);
                });
            });
        });

        frame.request_repaint();
    }
}
