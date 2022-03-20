mod instruction;

use std::{
    env::args,
    fmt::{self, Display},
    fs,
};

use rand::prelude::*;

use crate::instruction::Instruction;

#[derive(Debug, Clone, Copy)]
struct Stackframe {
    reg: [u8; 16],
    idx: u16,
    pc: u16,
}

#[derive(Debug)]
struct CHIP8 {
    current: Stackframe,
    stack: Vec<Stackframe>,
    mem: Box<[u8; 4096]>,
    display: [[bool; 64]; 32],
}

impl Display for CHIP8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CHIP8")
            .field("pc", &self.current.pc)
            .field("idx", &self.current.idx)
            .field("reg", &self.current.reg)
            .field("stack", &self.stack.len())
            .field(
                "next_instruction",
                &Instruction::from_bits(self.instruction_word_at(self.current.pc)),
            )
            .finish()?;
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
        mem[0x200..0x200 + instruction_section.len()].copy_from_slice(instruction_section);

        CHIP8 {
            current: Stackframe {
                reg: [0; 16],
                idx: 0,
                pc: 0x200,
            },
            stack: Vec::new(),
            mem,
            display: [[false; 64]; 32],
        }
    }

    fn advance(&mut self, amount: u16) -> Result<Continue, String> {
        self.current.pc += amount;
        Ok(Continue::Continue)
    }

    fn instruction_word_at(&self, addr: u16) -> u16 {
        u16::from_be_bytes([self.mem[addr as usize], self.mem[addr as usize + 1]])
    }

    fn step(&mut self) -> Result<Continue, String> {
        use crate::Continue::*;
        use Instruction::*;

        let instr_bits = self.instruction_word_at(self.current.pc);
        let instr = Instruction::from_bits(instr_bits)
            .ok_or(format!("Unknown instruction {:#x}", instr_bits))?;

        match instr {
            MOVE(x, y) => {
                self.current.reg[x as usize] = self.current.reg[y as usize];
                self.advance(2)
            }
            OR(x, y) => {
                self.current.reg[x as usize] |= self.current.reg[y as usize];
                self.advance(2)
            }
            AND(x, y) => {
                self.current.reg[x as usize] &= self.current.reg[y as usize];
                self.advance(2)
            }
            XOR(x, y) => {
                self.current.reg[x as usize] ^= self.current.reg[y as usize];
                self.advance(2)
            }
            ADDR(x, y) => {
                match self.current.reg[x as usize].checked_add(self.current.reg[y as usize]) {
                    Some(val) => {
                        self.current.reg[x as usize] = val;
                        self.current.reg[0xf] = 0;
                    }
                    None => {
                        self.current.reg[x as usize] = 0;
                        self.current.reg[0xf] = 1;
                    }
                }
                self.advance(2)
            }
            SUB(x, y) => {
                self.current.reg[x as usize] -= self.current.reg[y as usize];
                self.advance(2)
            }
            SHR(x, y) => {
                self.current.reg[0x0F] = self.current.reg[y as usize] & 1;
                self.current.reg[y as usize] = self.current.reg[x as usize] >> 1;
                self.advance(2)
            }
            SHL(x, y) => {
                self.current.reg[0x0F] = self.current.reg[y as usize] & 0xE0;
                self.current.reg[y as usize] = self.current.reg[x as usize] << 1;
                self.advance(2)
            }
            LOAD(x, n) => {
                self.current.reg[x as usize] = n;
                self.advance(2)
            }
            ADD(x, n) => {
                self.current.reg[x as usize] += n;
                self.advance(2)
            }
            // Subroutines
            CALL(addr) => {
                self.stack.push(self.current);
                self.current.reg = [0; 16];
                self.current.idx = 0;
                self.current.pc = addr;
                Ok(Continue)
            }
            RTS => {
                if let Some(sf) = self.stack.pop() {
                    self.current = sf;
                    self.advance(2)
                } else {
                    Err("Return from empty stack".to_string())
                }
            }
            // Jumps
            JUMP(ofs) => {
                self.current.pc = (self.current.pc & 0xF000) | (ofs & 0x0FFF);
                Ok(Continue)
            }
            JUMPI(addr) => {
                self.current.pc = addr + self.current.reg[0] as u16;
                Ok(Continue)
            }
            // Skip
            SKE(x, n) => {
                if self.current.reg[x as usize] == n {
                    self.advance(4)
                } else {
                    self.advance(2)
                }
            }
            SKNE(x, n) => {
                if self.current.reg[x as usize] != n {
                    self.advance(4)
                } else {
                    self.advance(2)
                }
            }
            SKRE(x, y) => {
                if self.current.reg[x as usize] != self.current.reg[y as usize] {
                    self.advance(4)
                } else {
                    self.advance(2)
                }
            }
            SKRNE(x, y) => {
                if self.current.reg[x as usize] != self.current.reg[y as usize] {
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
                self.current.idx += self.current.reg[x as usize] as u16;
                self.advance(2)
            }
            LOADI(addr) => {
                self.current.idx = addr;
                self.advance(2)
            }
            // Screen
            DRAW(reg_x, reg_y, n) => {
                let mut y = self.current.reg[reg_y as usize];

                self.current.reg[0x0F] = 0;
                for byte in
                    &self.mem[(self.current.idx as usize)..(self.current.idx + n as u16) as usize]
                {
                    let mut x = self.current.reg[reg_x as usize];
                    for bit in 0..7 {
                        let ref mut pixel = self.display[y as usize][x as usize];
                        if *pixel {
                            self.current.reg[0x0F] = 1
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
            BCD(x) => {
                let hundreds = self.current.reg[x as usize] / 100;
                let tens = (self.current.reg[x as usize] % 100) / 10;
                let ones = self.current.reg[x as usize] % 10;

                self.mem[self.current.idx as usize] = hundreds;
                self.mem[self.current.idx as usize + 1] = tens;
                self.mem[self.current.idx as usize + 2] = ones;

                self.advance(2)
            }
            RAND(x, n) => {
                let mut rng = rand::thread_rng();
                self.current.reg[x as usize] = rng.gen_range(0..n);
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
