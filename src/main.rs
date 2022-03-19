mod instruction;

use std::{env::args, fs, process::exit};

use crate::instruction::Instruction;

#[derive(Debug)]
struct CHIP8 {
    reg: [u8; 16],
    idx: u16,
    pc: u16,

    mem: Box<[u8; 4096]>,
}

impl CHIP8 {
    fn new(instruction_section: &[u8]) -> CHIP8 {
        let mut mem = Box::new([0; 4096]);
        mem[0x200..0x200 + instruction_section.len()].copy_from_slice(instruction_section);

        CHIP8 {
            reg: [0; 16],
            idx: 0,
            pc: 0x200,
            mem,
        }
    }

    fn step(&mut self) {
        let instr_bits =
            u16::from_be_bytes([self.mem[self.pc as usize], self.mem[self.pc as usize + 1]]);

        let instr_decode = Instruction::from_bits(instr_bits);

        match instr_decode {
            Some(Instruction::SYS(0)) => {
                println!("Null, exiting");
                exit(0);
            }
            Some(instr) => println!("Executing {:?}", instr),
            None => println!("Unknown instruction {:#x}", instr_bits),
        }

        self.pc += 2;
    }
}

fn main() {
    let args: Vec<String> = args().collect();
    assert_eq!(args.len(), 2);
    let filename = &args[1];

    println!("Reading file {}", filename);

    let instruction_mem: Vec<u8> = fs::read(filename).expect("open input file");

    {
        let instructions = instruction_mem
            .chunks_exact(2)
            .into_iter()
            .map(|a| u16::from_be_bytes([a[0], a[1]]))
            .map(|x| (x, Instruction::from_bits(x)))
            .collect::<Vec<_>>();

        println!("---");
        println!("Instructions: ");
        let mut addr = 0x200;
        for (bits, m_instruction) in instructions {
            if let Some(i) = m_instruction {
                println!("{:#x}: {:x} - {:?}", addr, bits, i);
            } else {
                println!("{:#x}: {:x} - ????", addr, bits);
            }

            addr += 2;
        }
    }

    {
        println!("---");
        println!("CPU");

        let mut cpu = CHIP8::new(&instruction_mem);
        loop {
            cpu.step();
        }
    }
}
