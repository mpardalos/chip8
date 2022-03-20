mod instruction;

use std::{
    cmp::min,
    env::args,
    fmt::{self, Display},
    fs,
};

use bitvec::prelude::*;
use rand::prelude::*;

use crate::instruction::Instruction;

#[derive(Debug)]
struct CHIP8 {
    stack: Vec<u16>,
    pc: u16,
    reg: [u8; 16],
    idx: u16,
    mem: Box<[u8; 4096]>,
    display: [BitArray<[u64; 1], Msb0>; 32],
}

impl Display for CHIP8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let instr = match Instruction::try_from(self.instruction_word_at(self.pc)) {
            Ok(i) => format!("{}", i),
            Err(e) => e,
        };

        write!(
            f,
            "CHIP8 | pc: {:#X} | {:<20} | idx: {:>3X} | reg: {:?} | stack: {}",
            self.pc,
            instr,
            self.idx,
            self.reg,
            self.stack.len()
        )?;
        writeln!(
            f,
            "\n┌────────────────────────────────────────────────────────────────┐"
        )?;
        for row in self.display {
            write!(f, "│")?;
            for pixel in row {
                if pixel {
                    write!(f, "█")?;
                } else {
                    write!(f, "·")?;
                }
            }
            write!(f, "│\n")?;
        }
        writeln!(
            f,
            "└────────────────────────────────────────────────────────────────┘"
        )?;
        Ok(())
    }
}

enum Continue {
    Continue,
    Stop,
}

impl CHIP8 {
    fn new(instruction_section: &[u8]) -> CHIP8 {
        let mut mem = Box::new([0; 4096]);
        mem[0] = 0xF0;
        mem[1] = 0x90;
        mem[2] = 0x90;
        mem[3] = 0x90;
        mem[4] = 0xF0;
        mem[5] = 0x20;
        mem[6] = 0x60;
        mem[7] = 0x20;
        mem[8] = 0x20;
        mem[9] = 0x70;
        mem[10] = 0xF0;
        mem[11] = 0x10;
        mem[12] = 0xF0;
        mem[13] = 0x80;
        mem[14] = 0xF0;
        mem[15] = 0xF0;
        mem[16] = 0x10;
        mem[17] = 0xF0;
        mem[18] = 0x10;
        mem[19] = 0xF0;
        mem[20] = 0x90;
        mem[21] = 0x90;
        mem[22] = 0xF0;
        mem[23] = 0x10;
        mem[24] = 0x10;
        mem[25] = 0xF0;
        mem[26] = 0x80;
        mem[27] = 0xF0;
        mem[28] = 0x10;
        mem[29] = 0xF0;
        mem[30] = 0xF0;
        mem[31] = 0x80;
        mem[32] = 0xF0;
        mem[33] = 0x90;
        mem[34] = 0xF0;
        mem[35] = 0xF0;
        mem[36] = 0x10;
        mem[37] = 0x20;
        mem[38] = 0x40;
        mem[39] = 0x40;
        mem[40] = 0xF0;
        mem[41] = 0x90;
        mem[42] = 0xF0;
        mem[43] = 0x90;
        mem[44] = 0xF0;
        mem[45] = 0xF0;
        mem[46] = 0x90;
        mem[47] = 0xF0;
        mem[48] = 0x10;
        mem[49] = 0xF0;
        mem[50] = 0xF0;
        mem[51] = 0x90;
        mem[52] = 0xF0;
        mem[53] = 0x90;
        mem[54] = 0x90;
        mem[55] = 0xE0;
        mem[56] = 0x90;
        mem[57] = 0xE0;
        mem[58] = 0x90;
        mem[59] = 0xE0;
        mem[60] = 0xF0;
        mem[61] = 0x80;
        mem[62] = 0x80;
        mem[63] = 0x80;
        mem[64] = 0xF0;
        mem[65] = 0xE0;
        mem[66] = 0x90;
        mem[67] = 0x90;
        mem[68] = 0x90;
        mem[69] = 0xE0;
        mem[70] = 0xF0;
        mem[71] = 0x80;
        mem[72] = 0xF0;
        mem[73] = 0x80;
        mem[74] = 0xF0;
        mem[75] = 0xF0;
        mem[76] = 0x80;
        mem[77] = 0xF0;
        mem[78] = 0x80;
        mem[79] = 0x80;

        mem[0x200..0x200 + instruction_section.len()].copy_from_slice(instruction_section);

        CHIP8 {
            reg: [0; 16],
            idx: 0,
            pc: 0x200,
            stack: Vec::new(),
            mem,
            display: [bitarr![u64, Msb0; 0; 64]; 32],
        }
    }

    fn advance(&mut self, amount: u16) -> Result<Continue, String> {
        self.pc += amount;
        Ok(Continue::Continue)
    }

    fn instruction_word_at(&self, addr: u16) -> u16 {
        u16::from_be_bytes([self.mem[addr as usize], self.mem[addr as usize + 1]])
    }

    fn step(&mut self) -> Result<Continue, String> {
        use crate::Continue::*;
        use Instruction::*;

        let instr = Instruction::try_from(self.instruction_word_at(self.pc))?;

        match instr {
            MOVE(x, y) => {
                self.reg[x as usize] = self.reg[y as usize];
                self.advance(2)
            }
            OR(x, y) => {
                self.reg[x as usize] |= self.reg[y as usize];
                self.advance(2)
            }
            AND(x, y) => {
                self.reg[x as usize] &= self.reg[y as usize];
                self.advance(2)
            }
            XOR(x, y) => {
                self.reg[x as usize] ^= self.reg[y as usize];
                self.advance(2)
            }
            ADDR(x, y) => {
                match self.reg[x as usize].checked_add(self.reg[y as usize]) {
                    Some(val) => {
                        self.reg[x as usize] = val;
                        self.reg[0xf] = 0;
                    }
                    None => {
                        self.reg[x as usize] = self.reg[x as usize].wrapping_add(self.reg[y as usize]);
                        self.reg[0xf] = 1;
                    }
                }
                self.advance(2)
            }
            SUB(x, y) => {
                self.reg[x as usize] -= self.reg[y as usize];
                self.advance(2)
            }
            SHR(x, y) => {
                self.reg[0x0F] = self.reg[y as usize] & 1;
                self.reg[y as usize] = self.reg[x as usize] >> 1;
                self.advance(2)
            }
            SHL(x, y) => {
                self.reg[0x0F] = self.reg[y as usize] & 0xE0;
                self.reg[y as usize] = self.reg[x as usize] << 1;
                self.advance(2)
            }
            LOAD(x, n) => {
                self.reg[x as usize] = n;
                self.advance(2)
            }
            ADD(x, n) => {
                self.reg[x as usize] = self.reg[x as usize].wrapping_add(n);
                self.advance(2)
            }
            // Subroutines
            CALL(addr) => {
                self.stack.push(self.pc);
                self.pc = addr;
                Ok(Continue)
            }
            RTS => {
                if let Some(pc) = self.stack.pop() {
                    self.pc = pc;
                    self.advance(2)
                } else {
                    Err("Return from empty stack".to_string())
                }
            }
            // Jumps
            JUMP(ofs) => {
                self.pc = (self.pc & 0xF000) | (ofs & 0x0FFF);
                Ok(Continue)
            }
            JUMPI(addr) => {
                self.pc = addr + self.reg[0] as u16;
                Ok(Continue)
            }
            // Skip
            SKE(x, n) => {
                if self.reg[x as usize] == n {
                    self.advance(4)
                } else {
                    self.advance(2)
                }
            }
            SKNE(x, n) => {
                if self.reg[x as usize] != n {
                    self.advance(4)
                } else {
                    self.advance(2)
                }
            }
            SKRE(x, y) => {
                if self.reg[x as usize] != self.reg[y as usize] {
                    self.advance(4)
                } else {
                    self.advance(2)
                }
            }
            SKRNE(x, y) => {
                if self.reg[x as usize] != self.reg[y as usize] {
                    self.advance(4)
                } else {
                    self.advance(2)
                }
            }
            // Memory
            STOR(x) => {
                for r in 0..=x {
                    self.mem[self.idx as usize] = self.reg[r as usize];
                    self.idx += 1;
                }

                self.advance(2)
            }
            READ(x) => {
                for r in 0..=x {
                    self.reg[r as usize] = self.mem[self.idx as usize];
                    self.idx += 1;
                }

                self.advance(2)
            }
            // Input
            // TODO: Implement input
            SKPR(_) => self.advance(2),
            SKUP(_) => self.advance(2),
            // Delays
            MOVED(_) => Err(format!("{:?}", instr)),
            KEYD(_) => Err(format!("{:?}", instr)),
            LOADD(_) => Err(format!("{:?}", instr)),

            // TODO: Implement sound
            LOADS(_) => self.advance(2),

            // Index register
            ADDI(x) => {
                self.idx += self.reg[x as usize] as u16;
                self.advance(2)
            }
            LOADI(addr) => {
                self.idx = addr;
                self.advance(2)
            }
            // Screen
            DRAW(reg_x, reg_y, n) => {
                let mut y = self.reg[reg_y as usize] as usize;
                let x = self.reg[reg_x as usize] as usize;
                let memidx = self.idx as usize;

                self.reg[0x0F] = 0;
                for byte in &self.mem[memidx..memidx + n as usize] {
                    if *byte != 0 {
                        self.reg[0x0F] = 1;
                    }

                    self.display[y][x..min(x + 8, 64)].store_be(*byte);

                    y += 1;
                }

                self.advance(2)
            }
            CLR => {
                self.display = [bitarr![u64, Msb0; 0; 64]; 32];
                self.advance(2)
            }
            // Other
            LDSPR(x) => {
                let val = self.reg[x as usize];
                if val > 15 {
                    Err(format!("LDSPR for {} > 15", val))
                } else {
                    self.idx = val as u16 * 5;
                    self.advance(2)
                }
            }
            BCD(x) => {
                let hundreds = self.reg[x as usize] / 100;
                let tens = (self.reg[x as usize] % 100) / 10;
                let ones = self.reg[x as usize] % 10;

                self.mem[self.idx as usize] = hundreds;
                self.mem[self.idx as usize + 1] = tens;
                self.mem[self.idx as usize + 2] = ones;

                self.advance(2)
            }
            RAND(x, n) => {
                let mut rng = rand::thread_rng();
                self.reg[x as usize] = rng.gen_range(0..n);
                self.advance(2)
            }
            SYS(0) => Ok(Stop),
            SYS(_) => Err("SYS".to_string()),
        }
    }
}

fn main() {
    let args: Vec<String> = args().collect();
    assert_eq!(args.len(), 2);
    let filename = &args[1];

    println!("Reading file {}", filename);

    let instruction_mem: Vec<u8> = fs::read(filename).expect("open input file");

    if false {
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
            println!("{}", cpu);
            wait_for_enter();
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
