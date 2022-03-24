use std::rc::{Rc, Weak};

use crate::instruction::Instruction;
use crate::instruction::Instruction::*;

type SrcProgram<'a> = &'a [(u16, Result<Instruction, String>)];
type Pc = u16;

pub fn analyze(prog: SrcProgram) {
    let starts = block_starts(prog);
    let blocks = starts.iter().map(|&start| (start, block_from(prog, start)));

    println!("Blocks:");
    for (start, m_block) in blocks {
        println!("{:#x}:", start);
        if let Some(block) = m_block {
            for instr in block.code {
                println!("  {}", instr);
            }

            print!("  -> ");
            for pc in block.next {
                print!("{:#x} ", pc);
            }
            println!("\n");
        } else {
            println!("  INVALID\n");
        }
    }
}

fn block_starts(prog: SrcProgram) -> Vec<Pc> {
    let mut pc = 0x200;
    let mut starts: Vec<Pc> = vec![0x200];
    for (_, m_instr) in prog {
        if let Ok(instr) = m_instr {
            let mut nexts = instr.next_pc(pc);
            if nexts.len() > 1 {
                starts.append(&mut nexts);
            }
        }

        pc += 2;
    }
    starts
}

fn addr_to_idx(addr: Pc) -> Option<usize> {
    Some(addr.checked_sub(0x200)? as usize / 2)
}

fn idx_to_addr(idx: usize) -> Pc {
    200 * (idx * 2) as Pc
}

fn block_from(prog: SrcProgram, start_pc: Pc) -> Option<Block> {
    let mut block = Block::new_empty();
    let mut pc = start_pc;
    loop {
        let instr = prog[addr_to_idx(pc)?].1.as_ref().ok()?;
        block.code.push(*instr);

        let nexts = instr.next_pc(pc);
        if nexts.len() > 1 {
            block.next = nexts;
            break;
        }

        pc += 2;
    }

    Some(block)
}

struct Block {
    code: Vec<Instruction>,

    prev: Vec<Pc>,
    next: Vec<Pc>,
}

impl Block {
    fn new_empty() -> Self {
        Block {
            code: Vec::new(),
            prev: Vec::new(),
            next: Vec::new(),
        }
    }
}

trait AnalyzeInstruction {
    fn next_pc(&self, this_pc: Pc) -> Vec<Pc>;
    fn branches(&self) -> bool;
}

impl AnalyzeInstruction for Instruction {
    fn next_pc(&self, this_pc: Pc) -> Vec<Pc> {
        match self {
            SKE(_, _) | SKPR(_) | SKUP(_) | SKNE(_, _) | SKRE(_, _) | SKRNE(_, _) => {
                vec![this_pc + 2, this_pc + 4]
            }
            _ => vec![this_pc + 2],
        }
    }

    fn branches(&self) -> bool {
        self.next_pc(0).len() > 1
    }
}
