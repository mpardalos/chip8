mod instruction;

use std::{
    env::args,
    fmt::{self, Display},
    fs,
};

use rand::prelude::*;

use crate::instruction::Instruction;

#[derive(Debug)]
struct CHIP8 {
    reg: [u8; 16],
    idx: u16,
    pc: u16,

    mem: Box<[u8; 4096]>,
    display: [[bool; 64]; 32],
}

impl Display for CHIP8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CHIP8")
            .field("pc", &self.pc)
            .field("idx", &self.idx)
            .field("reg", &self.reg)
            .field(
                "instruction",
                &Instruction::from_bits(self.instruction_word_at(self.pc)),
            )
            .finish()?;
        writeln!(
            f,
            "\n+----------------------------------------------------------------+"
        )?;
        for row in self.display {
            write!(f, "|")?;
            for pixel in row {
                if pixel {
                    write!(f, "X")?;
                } else {
                    write!(f, " ")?;
                }
            }
            write!(f, "|\n")?;
        }
        writeln!(
            f,
            "+----------------------------------------------------------------+"
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
        mem[0x200..0x200 + instruction_section.len()].copy_from_slice(instruction_section);

        CHIP8 {
            reg: [0; 16],
            idx: 0,
            pc: 0x200,
            mem,
            display: [[false; 64]; 32],
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

        let instr_bits = self.instruction_word_at(self.pc);
        let instr = Instruction::from_bits(instr_bits)
            .ok_or(format!("Unknown instruction {:#x}", instr_bits))?;

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
                        self.reg[x as usize] = 0;
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
                self.reg[x as usize] += n;
                self.advance(2)
            }
            // Subroutines
            CALL(_) => Err("Subroutines".to_string()),
            RTS => Err("Subroutines".to_string()),
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
            STOR(_) => Err("Memory".to_string()),
            READ(_) => Err("Memory".to_string()),
            // Input
            SKPR(_) => Err("Input".to_string()),
            SKUP(_) => Err("Input".to_string()),
            // Delays
            MOVED(_) => Err("Delays".to_string()),
            KEYD(_) => Err("Delays".to_string()),
            LOADD(_) => Err("Delays".to_string()),
            LOADS(_) => Err("Delays".to_string()),
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
                let mut y = self.reg[reg_y as usize];

                self.reg[0x0F] = 0;
                for byte in &self.mem[(self.idx as usize)..(self.idx + n as u16) as usize] {
                    let mut x = self.reg[reg_x as usize];
                    for bit in 0..7 {
                        let ref mut pixel = self.display[y as usize][x as usize];
                        if *pixel {
                            self.reg[0x0F] = 1
                        }
                        *pixel = (byte & (0xE0 >> bit)) != 0;
                        x += 1;
                    }
                    y += 1;
                }

                self.advance(2)
            }
            CLR => {
                self.display = [[false; 64]; 32];
                self.advance(2)
            }
            // Other
            LDSPR(_) => Err("LDSPR".to_string()),
            BCD(_) => Err("BCD".to_string()),
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
            println!("{}", cpu);
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
