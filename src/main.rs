mod cpu;
mod display;
mod instruction;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use std::{fs, time::Duration};

use clap::Parser;

use crate::cpu::{StepResult, CHIP8};
use crate::display::run_gui;
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
struct Args {
    /// Dump instructions loaded from rom file at the start
    #[clap(long)]
    dump_code: bool,

    /// Instructions per second
    #[clap(long, default_value_t = 1000)]
    ips: u64,

    /// Frames per second
    #[clap(long, default_value_t = 60)]
    fps: u64,

    /// Path to the rom file to load
    rom: String,
}

fn main() {
    let args = Args::parse();

    println!("Reading file {}", args.rom);
    let instruction_mem: Vec<u8> = fs::read(&args.rom).expect("open input file");

    if args.dump_code {
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
    } else {
        let cpu = Arc::new(Mutex::new(CHIP8::new(&instruction_mem)));
        let core_cpu = cpu.clone();

        let _cpu_thread = thread::spawn(move || {
            let mut ticker = Instant::now();
            loop {
                if core_cpu.lock().unwrap().step().unwrap() == StepResult::End {
                    break;
                };
                rate_limit(args.ips, &mut ticker);
            }
        });

        run_gui(args.fps, &cpu).unwrap();
    }
}
