use std::rc::{Rc, Weak};

use crate::instruction::Instruction;
use crate::instruction::Instruction::*;

type SrcProgram = Vec<(u16, Result<Instruction, String>)>;
type Pc = u16;

pub fn analyze(prog: SrcProgram) {
    println!("Starts:");
    for addr in block_starts(prog) {
        println!("  -> {:#x}", addr);
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

struct Block {
    code: Vec<Instruction>,

    prev: Vec<Weak<Block>>,

    // TODO: Leaks? Maybe it's fine since this does not run for too long anyways
    next: Vec<Rc<Block>>,
}

trait AnalyzeInstruction {
    fn next_pc(&self, this_pc: Pc) -> Vec<Pc>;
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
}
