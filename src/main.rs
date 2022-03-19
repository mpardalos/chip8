use std::{env::args, fs};

type Addr = u16;
// type MemVal = u16;
type Reg = u8;
type RegVal = u8;
type ShortVal = u8;

#[derive(Debug)]
enum Instruction {
    SYS(u16),
    CLR,
    RTS,
    JUMP(Addr),
    CALL(Addr),
    SKE(Reg, RegVal),
    SKNE(Reg, RegVal),
    SKRE(Reg, Reg),
    LOAD(Reg, RegVal),
    ADD(Reg, RegVal),
    MOVE(Reg, Reg),
    OR(Reg, Reg),
    AND(Reg, Reg),
    XOR(Reg, Reg),
    ADDR(Reg, Reg),
    SUB(Reg, Reg),
    SHR(Reg, Reg),
    SHL(Reg, Reg),
    SKRNE(Reg, RegVal),
    LOADI(Addr),
    JUMPI(Addr),
    RAND(Reg, RegVal),
    DRAW(ShortVal, Reg, Reg),
    SKPR(Reg),
    SKUP(Reg),
    MOVED(Reg),
    KEYD(Reg),
    LOADD(Reg),
    LOADS(Reg),
    ADDI(Reg),
    LDSPR(Reg),
    BCD(Reg),
    STOR(Reg),
    READ(Reg),
}

fn addr(x: u16) -> Addr {
    x & 0x0FFF
}

fn imm(x: u16) -> RegVal {
    (x & 0x00FF) as RegVal
}

fn r1(x: u16) -> Reg {
    ((x & 0x0F00) >> 8) as Reg
}

fn r2(x: u16) -> Reg {
    ((x & 0x00F0) >> 4) as Reg
}

impl Instruction {
    fn from_bits(x: u16) -> Option<Instruction> {
        use Instruction::*;
        match x & 0xF000 {
            0x0000 => match x {
                0x00E0 => Some(CLR),
                0x00EE => Some(RTS),
                _ => Some(SYS(addr(x))),
            },
            0x1000 => Some(JUMP(addr(x))),
            0x2000 => Some(CALL(addr(x))),
            0x3000 => Some(SKE(r1(x), imm(x))),
            0x4000 => Some(SKNE(r1(x), imm(x))),
            0x5000 => match x & 0x000F {
                0x0 => Some(SKRE(r1(x), r2(x))),
                _ => None,
            },
            0x6000 => Some(LOAD(r1(x), imm(x))),
            0x7000 => Some(ADD(r1(x), imm(x))),
            0x8000 => match x & 0x000F {
                0x0 => Some(MOVE(r1(x), r2(x))),
                0x1 => Some(OR(r1(x), r2(x))),
                0x2 => Some(AND(r1(x), r2(x))),
                0x3 => Some(XOR(r1(x), r2(x))),
                0x4 => Some(ADDR(r1(x), r2(x))),
                0x5 => Some(SUB(r1(x), r2(x))),
                0x6 => Some(SHR(r1(x), r2(x))),
                0xE => Some(SHL(r1(x), r2(x))),
                _ => None,
            },
            0x9000 => match x & 0x000F {
                0x0 => Some(SKRNE(r1(x), r2(x))),
                _ => None,
            },
            0xA000 => Some(LOADI(addr(x))),
            0xB000 => Some(JUMPI(addr(x))),
            0xC000 => Some(RAND(r1(x), imm(x))),
            0xD000 => Some(DRAW(r1(x), r2(x), (x & 0x000F) as ShortVal)),
            0xE000 => match x & 0x00FF {
                0x9E => Some(SKPR(r1(x))),
                0xA1 => Some(SKUP(r1(x))),
                _ => None,
            },
            0xF000 => match x & 0x00FF {
                0x07 => Some(MOVED(r1(x))),
                0x0A => Some(KEYD(r1(x))),
                0x15 => Some(LOADD(r1(x))),
                0x18 => Some(LOADS(r1(x))),
                0x1E => Some(ADDI(r1(x))),
                0x29 => Some(LDSPR(r1(x))),
                0x33 => Some(BCD(r1(x))),
                0x55 => Some(STOR(r1(x))),
                0x65 => Some(READ(r1(x))),
                _ => None,
            },
            _ => None,
        }
    }
}

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
