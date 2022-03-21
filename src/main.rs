mod cpu;
mod instruction;

use std::{fs, thread::sleep, time::Duration};

use clap::Parser;

use crate::cpu::{CHIP8, Continue};
use crate::instruction::Instruction;

#[derive(Parser, Debug)]
struct Args {
    /// Use terminal rendering
    #[clap(short, long)]
    term: bool,

    /// Dump instructions loaded from rom file at the start
    #[clap(long)]
    dump_code: bool,

    /// Path to the rom file to load
    rom: String,
}

impl Args {
    fn should_run(&self) -> bool {
        !self.dump_code
    }
}

fn main() {
    let args = Args::parse();
    if args.term {
        panic!("GUI not implemented")
    }

    println!("Reading file {}", args.rom);
    let instruction_mem: Vec<u8> = fs::read(&args.rom).expect("open input file");

    if args.dump_code {
        let instructions = instruction_mem
            .chunks_exact(2)
            .into_iter()
            .map(|a| u16::from_be_bytes([a[0], a[1]]))
            .map(|x| (x, Instruction::try_from(x)))
            .collect::<Vec<_>>();

        println!("---");
        println!("Instructions: ");
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

    if args.should_run() {
        println!("---");
        println!("CPU");

        let mut cpu = CHIP8::new(&instruction_mem);
        loop {
            println!("{}", cpu);
            // wait_for_enter();
            sleep(Duration::from_millis(5));
            clear_screen();
            match cpu.step() {
                Ok(Continue::Stop) => {
                    println!("Done!");
                    break;
                }
                Ok(Continue::Continue) => {}
                Err(err) => {
                    println!("Error: {}", err);
                    break;
                }
            }
        }
    }
}

fn wait_for_enter() {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}

fn clear_screen() {
    print!("\x1B[2J\n");
}
