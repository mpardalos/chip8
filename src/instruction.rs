use std::fmt;

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

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Instruction::*;
        match self {
            CLR           => write!(f, "CLR"),
            RTS           => write!(f, "RTS"),
            SYS(addr)     => write!(f, "SYS   {:#x}", addr),
            JUMP(addr)    => write!(f, "JUMP  {:#x}", addr),
            CALL(addr)    => write!(f, "CALL  {:#x}", addr),
            LOADI(addr)   => write!(f, "LOADI {:#x}", addr),
            JUMPI(addr)   => write!(f, "JUMPI {:#x}", addr),
            SKE(x, n)     => write!(f, "SKE   v{:<2}, {:#x}", x, n),
            SKNE(x, n)    => write!(f, "SKNE  v{:<2}, {:#x}", x, n),
            LOAD(x, n)    => write!(f, "LOAD  v{:<2}, {:#x}", x, n),
            ADD(x, n)     => write!(f, "ADD   v{:<2}, {:#x}", x, n),
            RAND(x, n)    => write!(f, "RAND  v{:<2}, {:#x}", x, n),
            SKRE(x, y)    => write!(f, "SKRE  v{:<2}, v{}", x, y),
            MOVE(x, y)    => write!(f, "MOVE  v{:<2}, v{}", x, y),
            OR(x, y)      => write!(f, "OR    v{:<2}, v{}", x, y),
            AND(x, y)     => write!(f, "AND   v{:<2}, v{}", x, y),
            XOR(x, y)     => write!(f, "XOR   v{:<2}, v{}", x, y),
            ADDR(x, y)    => write!(f, "ADDR  v{:<2}, v{}", x, y),
            SUB(x, y)     => write!(f, "SUB   v{:<2}, v{}", x, y),
            SHR(x, y)     => write!(f, "SHR   v{:<2}, v{}", x, y),
            SHL(x, y)     => write!(f, "SHL   v{:<2}, v{}", x, y),
            SKRNE(x, y)   => write!(f, "SKRNE v{:<2}, v{}", x, y),
            DRAW(x, y, n) => write!(f, "DRAW  v{:<2}, v{}, {:#x}", x, y, n),
            SKPR(x)       => write!(f, "SKPR  v{:<2}", x),
            SKUP(x)       => write!(f, "SKUP  v{:<2}", x),
            MOVED(x)      => write!(f, "MOVED v{:<2}", x),
            KEYD(x)       => write!(f, "KEYD  v{:<2}", x),
            LOADD(x)      => write!(f, "LOADD v{:<2}", x),
            LOADS(x)      => write!(f, "LOADS v{:<2}", x),
            ADDI(x)       => write!(f, "ADDI  v{:<2}", x),
            LDSPR(x)      => write!(f, "LDSPR v{:<2}", x),
            BCD(x)        => write!(f, "BCD   v{:<2}", x),
            STOR(x)       => write!(f, "STOR  v{:<2}", x),
            READ(x)       => write!(f, "READ  v{:<2}", x),
        }
    }
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

impl TryFrom<u16> for Instruction {
    type Error = String;

    fn try_from(x: u16) -> Result<Self, Self::Error> {
        use Instruction::*;
        match x & 0xF000 {
            0x0000 => match x {
                0x00E0 => Ok(CLR),
                0x00EE => Ok(RTS),
                _ => Ok(SYS(addr(x))),
            },
            0x1000 => Ok(JUMP(addr(x))),
            0x2000 => Ok(CALL(addr(x))),
            0x3000 => Ok(SKE(r1(x), imm(x))),
            0x4000 => Ok(SKNE(r1(x), imm(x))),
            0x5000 => match x & 0x000F {
                0x0 => Ok(SKRE(r1(x), r2(x))),
                _ => Err(format!("Invalid Instruction: {:#x}", x)),
            },
            0x6000 => Ok(LOAD(r1(x), imm(x))),
            0x7000 => Ok(ADD(r1(x), imm(x))),
            0x8000 => match x & 0x000F {
                0x0 => Ok(MOVE(r1(x), r2(x))),
                0x1 => Ok(OR(r1(x), r2(x))),
                0x2 => Ok(AND(r1(x), r2(x))),
                0x3 => Ok(XOR(r1(x), r2(x))),
                0x4 => Ok(ADDR(r1(x), r2(x))),
                0x5 => Ok(SUB(r1(x), r2(x))),
                0x6 => Ok(SHR(r1(x), r2(x))),
                0xE => Ok(SHL(r1(x), r2(x))),
                _ => Err(format!("Invalid Instruction: {:#x}", x)),
            },
            0x9000 => match x & 0x000F {
                0x0 => Ok(SKRNE(r1(x), r2(x))),
                _ => Err(format!("Invalid Instruction: {:#x}", x)),
            },
            0xA000 => Ok(LOADI(addr(x))),
            0xB000 => Ok(JUMPI(addr(x))),
            0xC000 => Ok(RAND(r1(x), imm(x))),
            0xD000 => Ok(DRAW(r1(x), r2(x), (x & 0x000F) as ShortVal)),
            0xE000 => match x & 0x00FF {
                0x9E => Ok(SKPR(r1(x))),
                0xA1 => Ok(SKUP(r1(x))),
                _ => Err(format!("Invalid Instruction: {:#x}", x)),
            },
            0xF000 => match x & 0x00FF {
                0x07 => Ok(MOVED(r1(x))),
                0x0A => Ok(KEYD(r1(x))),
                0x15 => Ok(LOADD(r1(x))),
                0x18 => Ok(LOADS(r1(x))),
                0x1E => Ok(ADDI(r1(x))),
                0x29 => Ok(LDSPR(r1(x))),
                0x33 => Ok(BCD(r1(x))),
                0x55 => Ok(STOR(r1(x))),
                0x65 => Ok(READ(r1(x))),
                _ => Err(format!("Invalid Instruction: {:#x}", x)),
            },
            _ => Err(format!("Invalid Instruction: {:#x}", x)),
        }
    }
}
