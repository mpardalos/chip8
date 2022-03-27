mod analyze;
mod cpu;
mod gui;
mod instruction;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use std::{fs, time::Duration};

use analyze::analyze;
use clap::Parser;

use crate::cpu::{Chip8, Chip8IO, StepResult};
use crate::gui::Chip8Gui;
use crate::instruction::Instruction;

/// Call this in a loop to limit how many times per second the loop runs
pub fn rate_limit(ticks_per_sec: u64, ticker: &mut Instant) -> (Duration, Duration) {
    let last_tick = *ticker;
    let task_end = Instant::now();
    let busy_elapsed = task_end - *ticker;
    let target = Duration::from_nanos(1_000_000_000 / ticks_per_sec);

    if target > busy_elapsed {
        thread::sleep(target - busy_elapsed);
    }

    let loop_end = Instant::now();
    let full_elapsed = loop_end - last_tick;

    *ticker = loop_end;

    (busy_elapsed, full_elapsed)
}

#[derive(Parser, Debug)]
enum Args {
    /// What can we learn from the ROM file?
    Analyze {
        /// Path to the rom file to load
        rom: String,
    },
    /// Dump instructions
    Dump {
        /// Path to the rom file to load
        rom: String,
    },
    /// Run the ROM
    Run {
        /// Instructions per second
        #[clap(long, default_value_t = 1000)]
        ips: u64,

        /// Output I/O debug information to the terminal
        #[clap(long)]
        debug_io: bool,

        /// Output CPU debug information to the terminal
        #[clap(long)]
        debug_cpu: bool,

        /// Path to the rom file to load
        rom: String,
    },
}

impl Args {
    fn rom_bytes(&self) -> Vec<u8> {
        let rom = match self {
            Args::Analyze { rom, .. } => rom,
            Args::Run { rom, .. } => rom,
            Args::Dump { rom, .. } => rom,
        };

        println!("Reading file {}", rom);
        fs::read(&rom).expect("open input file")
    }
}

fn main() {
    let args = Args::parse();
    let instruction_mem: Vec<u8> = args.rom_bytes();
    match args {
        Args::Dump { .. } => {
            let instructions = instruction_mem
                .chunks_exact(2)
                .into_iter()
                .map(|a| u16::from_be_bytes([a[0], a[1]]))
                .map(|x| (x, Instruction::try_from(x)))
                .collect::<Vec<_>>();

            println!("Initial RAM: ");
            let mut addr = 0x200;
            for (bits, m_instruction) in instructions {
                if let Ok(i) = m_instruction {
                    println!("{:#x}: {:x} - {}", addr, bits, i);
                } else {
                    println!("{:#x}: {:x} - ????", addr, bits);
                }

                addr += 2;
            }
        }

        Args::Run {
            debug_cpu,
            debug_io,
            ips,
            ..
        } => {
            let io = Arc::new(Mutex::new(Chip8IO::new()));
            let cpu = Arc::new(Mutex::new(Chip8::new(&instruction_mem, io.clone())));
            let gui = Chip8Gui::new(cpu.clone(), io.clone());

            if debug_io {
                let debug_io = io.clone();
                let _debug_thread = thread::spawn(move || {
                    let mut ticker = Instant::now();
                    loop {
                        println!("{}", debug_io.lock().unwrap());
                        rate_limit(2, &mut ticker);
                    }
                });
            }

            let _cpu_thread = thread::spawn(move || {
                let mut ticker = Instant::now();
                loop {
                    if cpu.lock().unwrap().step().unwrap() == StepResult::End {
                        break;
                    };

                    if debug_cpu {
                        println!("{}", cpu.lock().unwrap());
                    }
                    rate_limit(ips, &mut ticker);
                }
            });

            gui.run();
        }

        Args::Analyze { .. } => {
            analyze(
                &instruction_mem
                    .chunks_exact(2)
                    .into_iter()
                    .map(|a| u16::from_be_bytes([a[0], a[1]]))
                    .map(|x| (x, Instruction::try_from(x)))
                    .collect::<Vec<_>>(),
            );
        }
    };
}
