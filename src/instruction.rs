pub type Addr = u16;
// type MemVal = u16;
pub type Reg = u8;
pub type RegVal = u8;
pub type ShortVal = u8;

#[derive(Debug)]
pub enum Instruction {
    /// Opcode: 0nnn
    SYS(u16),
    /// Opcode: 00E0
    CLR,
    /// Opcode: 00EE
    RTS,
    /// Opcode: 1nnn
    JUMP(Addr),
    /// Opcode: 2nnn
    CALL(Addr),
    /// Opcode: 3xnn
    SKE(Reg, RegVal),
    /// Opcode: 4xnn
    SKNE(Reg, RegVal),
    /// Opcode: 5xy0
    SKRE(Reg, Reg),
    /// Opcode: 6xnn
    LOAD(Reg, RegVal),
    /// Opcode: 7xnn
    ADD(Reg, RegVal),
    /// Opcode: 8xy0
    MOVE(Reg, Reg),
    /// Opcode: 8xy1
    OR(Reg, Reg),
    /// Opcode: 8xy2
    AND(Reg, Reg),
    /// Opcode: 8xy3
    XOR(Reg, Reg),
    /// Opcode: 8xy4
    ADDR(Reg, Reg),
    /// Opcode: 8xy5
    SUB(Reg, Reg),
    /// Opcode: 8xy6
    SHR(Reg, Reg),
    /// Opcode: 8xyE
    SHL(Reg, Reg),
    /// Opcode: 9xy0
    SKRNE(Reg, Reg),
    /// Opcode: Annn
    LOADI(Addr),
    /// Opcode: Bnnn
    JUMPI(Addr),
    /// Opcode: Cxnn
    RAND(Reg, RegVal),
    /// Opcode: Dxyn
    DRAW(ShortVal, Reg, Reg),
    /// Opcode: Ex9E
    SKPR(Reg),
    /// Opcode: ExA1
    SKUP(Reg),
    /// Opcode: Fx07
    MOVED(Reg),
    /// Opcode: Fx0A
    KEYD(Reg),
    /// Opcode: Fx15
    LOADD(Reg),
    /// Opcode: Fx18
    LOADS(Reg),
    /// Opcode: Fx1E
    ADDI(Reg),
    /// Opcode: Fx29
    LDSPR(Reg),
    /// Opcode: Fx33
    BCD(Reg),
    /// Opcode: Fx55
    STOR(Reg),
    /// Opcode: Fx65
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
    pub fn from_bits(x: u16) -> Option<Instruction> {
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
