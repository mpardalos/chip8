use std::{
    fmt::{self, Display},
    sync::{Arc, Mutex},
    time,
};

use phf::{phf_map, phf_ordered_map};
use rand::prelude::*;

use crate::instruction::Instruction;
use Instruction::*;

pub const DISPLAY_ROWS: usize = 32;
pub const DISPLAY_COLS: usize = 64;

#[derive(Debug)]
pub struct Chip8IO {
    pub keystate: [bool; 16],
    pub display: [[bool; DISPLAY_COLS]; DISPLAY_ROWS],
}

/*******************\
* Keypad mapping    *
*                   *
* QWERTY  | Chip-8  *
* --------+-------- *
* 1 2 3 4 | 1 2 3 C *
* Q W E R | 4 5 6 D *
* A S D F | 7 8 9 E *
* Z X C V | A 0 B F *
\*******************/
pub const KEYPAD_TO_QWERTY: phf::OrderedMap<u8, char> = phf_ordered_map! {
  0x1u8 => '1',
  0x2u8 => '2',
  0x3u8 => '3',
  0xCu8 => '4',

  0x4u8 => 'Q',
  0x5u8 => 'W',
  0x6u8 => 'E',
  0xDu8 => 'R',

  0x7u8 => 'A',
  0x8u8 => 'S',
  0x9u8 => 'D',
  0xEu8 => 'F',

  0xAu8 => 'Z',
  0x0u8 => 'X',
  0xBu8 => 'C',
  0xFu8 => 'V',
};

impl Chip8IO {
    pub fn new() -> Chip8IO {
        Chip8IO {
            keystate: [false; 16],
            display: [[false; DISPLAY_COLS]; DISPLAY_ROWS],
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

#[derive(Debug)]
pub struct Chip8 {
    pub stack: Vec<u16>,
    pub pc: u16,
    pub reg: [u8; 16],
    pub idx: u16,
    pub delay: u8,
    tick: time::Instant,
    init_mem: Box<[u8; 4096]>,
    pub mem: Box<[u8; 4096]>,
    pub io: Arc<Mutex<Chip8IO>>,

    pub paused: bool,
}

/// Outcome of one step of execution
#[derive(PartialEq, Eq)]
pub enum StepResult {
    /// Program continues. Bool specifies whether the display was updated
    Continue(bool),

    /// Endlessly looping
    Loop,

    /// Program ends.
    End,
}

fn wkey(f: &mut fmt::Formatter<'_>, keystate: [bool; 16], key: u8) -> fmt::Result {
    if keystate[key as usize] {
        write!(f, "{:X}", key)
    } else {
        write!(f, "█")
    }
}

impl Display for Chip8IO {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        wkey(f, self.keystate, 0x1)?;
        wkey(f, self.keystate, 0x2)?;
        wkey(f, self.keystate, 0x3)?;
        wkey(f, self.keystate, 0xC)?;
        writeln!(f)?;
        wkey(f, self.keystate, 0x4)?;
        wkey(f, self.keystate, 0x5)?;
        wkey(f, self.keystate, 0x6)?;
        wkey(f, self.keystate, 0xD)?;
        writeln!(f)?;
        wkey(f, self.keystate, 0x7)?;
        wkey(f, self.keystate, 0x8)?;
        wkey(f, self.keystate, 0x9)?;
        wkey(f, self.keystate, 0xE)?;
        writeln!(f)?;
        wkey(f, self.keystate, 0xA)?;
        wkey(f, self.keystate, 0x0)?;
        wkey(f, self.keystate, 0xB)?;
        wkey(f, self.keystate, 0xF)?;
        writeln!(f)?;

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

impl Display for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let instr = match self.current_instruction() {
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
        Ok(())
    }
}

impl Chip8 {
    pub fn new(instruction_section: &[u8], io: Arc<Mutex<Chip8IO>>, paused: bool) -> Chip8 {
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

        Chip8 {
            reg: [0; 16],
            idx: 0,
            pc: 0x200,
            stack: Vec::new(),
            delay: 0,
            tick: time::Instant::now(),
            init_mem: mem.clone(),
            mem,
            io,
            paused,
        }
    }

    fn advance(&mut self, amount: u16) -> Result<StepResult, String> {
        self.pc += amount;
        Ok(StepResult::Continue(false))
    }

    pub fn reset(&mut self) {
        self.reg = [0; 16];
        self.idx = 0;
        self.pc = 0x200;
        self.stack = Vec::new();
        self.delay = 0;
        self.tick = time::Instant::now();
        self.mem = self.init_mem.clone();
        self.io.lock().unwrap().reset();
    }

    pub fn current_instruction(&self) -> Result<Instruction, String> {
        Instruction::try_from(u16::from_be_bytes([
            self.mem[self.pc as usize],
            self.mem[self.pc as usize + 1],
        ]))
    }

    pub fn step(&mut self) -> Result<StepResult, String> {
        if self.paused {
            return Ok(StepResult::Continue(false));
        }

        if time::Instant::now() - self.tick > time::Duration::from_millis(016) {
            self.delay = self.delay.saturating_sub(1);
            self.tick = time::Instant::now();
        }

        match self.current_instruction()? {
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
                        self.reg[x as usize] =
                            self.reg[x as usize].wrapping_add(self.reg[y as usize]);
                        self.reg[0xf] = 1;
                    }
                }
                self.advance(2)
            }
            SUB(x, y) => {
                self.reg[x as usize] = self.reg[x as usize].wrapping_sub(self.reg[y as usize]);
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
                if addr == self.pc {
                    Ok(StepResult::Loop)
                } else {
                    self.stack.push(self.pc);
                    self.pc = addr;
                    Ok(StepResult::Continue(false))
                }
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
                let next_pc = (self.pc & 0xF000) | (ofs & 0x0FFF);
                if next_pc == self.pc {
                    Ok(StepResult::Loop)
                } else {
                    self.pc = next_pc;
                    Ok(StepResult::Continue(false))
                }
            }
            JUMPI(addr) => {
                let next_pc = addr + self.reg[0] as u16;
                if next_pc == self.pc {
                    Ok(StepResult::Loop)
                } else {
                    self.pc = next_pc;
                    Ok(StepResult::Continue(false))
                }
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
            SKPR(x) => {
                let keyidx: usize = self.reg[x as usize] as usize;
                let pressed = *self
                    .io
                    .lock()
                    .unwrap()
                    .keystate
                    .get(keyidx)
                    .unwrap_or(&false);
                if pressed {
                    self.advance(4)
                } else {
                    self.advance(2)
                }
            }
            SKUP(x) => {
                let keyidx: usize = self.reg[x as usize] as usize;
                let pressed = *self
                    .io
                    .lock()
                    .unwrap()
                    .keystate
                    .get(keyidx)
                    .unwrap_or(&false);
                if !pressed {
                    self.advance(4)
                } else {
                    self.advance(2)
                }
            }
            KEYD(x) => {
                let keystate = self.io.lock().unwrap().keystate;
                for (key, &pressed) in keystate.iter().enumerate() {
                    if pressed {
                        self.reg[x as usize] = key as u8;
                        let _ = self.advance(2);
                        break;
                    }
                }
                Ok(StepResult::Continue(false))
            }

            // Sound
            // TODO: Implement sound
            LOADS(_) => self.advance(2),

            // Delays
            MOVED(x) => {
                self.reg[x as usize] = self.delay;
                self.advance(2)
            }
            LOADD(x) => {
                self.delay = self.reg[x as usize];
                self.advance(2)
            }

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
            DRAW(x, y, n) => {
                let mut row = self.reg[y as usize] as usize;
                let memidx = self.idx as usize;

                {
                    // Lock IO here
                    let display = &mut self.io.lock().unwrap().display;
                    self.reg[0x0F] = 0;
                    for byte in &self.mem[memidx..memidx + n as usize] {
                        let mut col = self.reg[x as usize] as usize;
                        for bitidx in 0..8 {
                            let bit = (byte & (1 << (7 - bitidx))) != 0;
                            if display[row % DISPLAY_ROWS][col % DISPLAY_COLS] & bit {
                                self.reg[0x0F] = 1;
                            }

                            display[row % DISPLAY_ROWS][col % DISPLAY_COLS] ^= bit;
                            col += 1;
                        }

                        row += 1;
                    }
                }

                let _ = self.advance(2);
                Ok(StepResult::Continue(true))
            }
            CLR => {
                self.io.lock().unwrap().display = [[false; 64]; 32];
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
            SYS(0) => Ok(StepResult::End),
            SYS(_) => Err("SYS".to_string()),
        }
    }

    #[cfg(test)]
    fn new_test(code: &[Instruction]) -> Chip8 {
        let mut instr_ram: Vec<u8> = Vec::new();
        for instr in code {
            let [high, low] = u16::from(*instr).to_be_bytes();
            instr_ram.push(high);
            instr_ram.push(low);
        }
        Self::new(&instr_ram, Arc::new(Mutex::new(Chip8IO::new())), false)
    }

    #[cfg(test)]
    fn run_to_end(&mut self) {
        loop {
            match self.step() {
                Ok(StepResult::Continue(_)) => {}
                _ => break,
            }
        }
    }
}

#[test]
fn load() {
    let mut cpu = Chip8::new_test(&[LOAD(0, 10)]);
    cpu.run_to_end();

    assert_eq!(cpu.reg[0], 10);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn skne_not() {
    let mut cpu = Chip8::new_test(&[SKNE(0, 10), LOAD(1, 42)]);
    cpu.reg[0] = 10;
    cpu.run_to_end();

    assert_eq!(cpu.reg[1], 42);
    assert_eq!(cpu.pc, 0x204);
}

#[test]
fn skne_yes() {
    let mut cpu = Chip8::new_test(&[SKNE(0, 10), LOAD(1, 42)]);
    cpu.reg[0] = 110;
    cpu.reg[1] = 142;
    cpu.run_to_end();

    assert_eq!(cpu.reg[1], 142);
    assert_eq!(cpu.pc, 0x204);
}

#[test]
fn call_rts() {
    let mut cpu = Chip8::new_test(&[
        CALL(0x210), // 0x200
        LOAD(0, 42), // 0x202
        SYS(0),      // 0x204
        SYS(0),      // 0x206
        SYS(0),      // 0x208
        SYS(0),      // 0x20a
        SYS(0),      // 0x20c
        SYS(0),      // 0x20e
        RTS,         // 0x210
    ]);
    cpu.run_to_end();

    assert_eq!(cpu.reg[0], 42);
    assert!(cpu.stack.is_empty());
}

#[test]
fn rand_limit() {
    for _ in 0..100 {
        let mut cpu = Chip8::new_test(&[RAND(0, 10)]);
        cpu.run_to_end();
        assert!(cpu.reg[0] < 10);
    }
}

#[test]
fn skup_pressed() {
    let mut cpu = Chip8::new_test(&[SKUP(0), LOAD(1, 42)]);
    cpu.reg[0] = 5;
    cpu.io.lock().unwrap().keystate[5] = true;
    cpu.reg[1] = 0;
    cpu.run_to_end();

    assert_eq!(cpu.reg[1], 42);
}

#[test]
fn skup_up() {
    let mut cpu = Chip8::new_test(&[SKUP(0), LOAD(1, 42)]);
    cpu.reg[0] = 5;
    cpu.io.lock().unwrap().keystate[5] = false;
    cpu.reg[1] = 0;
    cpu.run_to_end();

    assert_eq!(cpu.reg[1], 0);
}

#[test]
fn draw_xor_true_begin() {
    let mut cpu = Chip8::new_test(&[DRAW(0, 1, 2)]);
    cpu.reg[0] = 0;
    cpu.reg[1] = 0;
    cpu.idx = 0x300;
    cpu.mem[0x300] = 0xFF;
    cpu.mem[0x301] = 0xFF;
    cpu.io.lock().unwrap().display[0][0] = true;
    cpu.run_to_end();

    assert_eq!(cpu.reg[0xF], 1);
}

#[test]
fn draw_xor_true_end() {
    let mut cpu = Chip8::new_test(&[DRAW(0, 1, 2)]);
    cpu.reg[0] = 0;
    cpu.reg[1] = 0;
    cpu.idx = 0x300;
    cpu.mem[0x300] = 0xFF;
    cpu.mem[0x301] = 0xFF;
    cpu.io.lock().unwrap().display[1][7] = true;
    cpu.run_to_end();

    assert_eq!(cpu.reg[0xF], 1);
}

#[test]
fn draw_xor_false() {
    let mut cpu = Chip8::new_test(&[DRAW(0, 1, 2)]);
    cpu.reg[0] = 0;
    cpu.reg[1] = 0;
    cpu.idx = 0x300;
    cpu.mem[0x300] = 0xFF;
    cpu.mem[0x301] = 0xFF;
    // cpu.io.lock().unwrap().display[0][0] = false;
    cpu.run_to_end();

    assert_eq!(cpu.reg[0xF], 0);
}
