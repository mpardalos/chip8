mod instruction;

use std::{env::args, fs};

use crate::instruction::Instruction;

fn main() {
    let args: Vec<String> = args().collect();
    assert_eq!(args.len(), 2);
    let filename = &args[1];

    println!("Reading file {}", filename);

    let start_mem: Vec<u8> = fs::read(filename).expect("open input file");

    let instructions: Vec<Instruction> = start_mem
        .chunks_exact(2)
        .into_iter()
        .map(|a| u16::from_be_bytes([a[0], a[1]]))
        .flat_map(Instruction::from_bits)
        .collect();

    let mut addr = 0x200;
    for instruction in instructions {
        println!("{:#x}: {:?}", addr, instruction);
        addr += 2;
    }
}
