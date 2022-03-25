use std::collections::HashMap;

use crate::instruction::Instruction;
use crate::instruction::Instruction::*;

type SrcProgram<'a> = &'a [(u16, Result<Instruction, String>)];
type Pc = u16;

struct Cfg {
    contents: HashMap<Pc, Block>,
}

impl Cfg {
    fn from_rom(rom: impl Iterator<Item = Option<Instruction>>) -> Cfg {
        let mut pc = 0x200;
        let mut contents: HashMap<Pc, Block> = rom
            .map(|m_instr| {
                let this_pc = pc;
                pc += 2;
                if let Some(instr) = m_instr {
                    (this_pc, Block::from_single(this_pc, instr))
                } else {
                    (this_pc, Block::new_empty())
                }
            })
            .collect();

        // Fill in prev pointers
        for (pc, block) in contents.clone().iter_mut() {
            for next_pc in block.next.iter_mut() {
                if let Some(next) = contents.get_mut(&next_pc) {
                    next.prev.push(*pc);
                } else {
                    println!("invalid next pointer: {:#x}", next_pc);
                }
            }
        }

        let cfg = Cfg { contents };
        cfg.assert_valid();
        cfg
    }

    fn debug_print(&self) {
        let mut block_pcs = self.contents.keys().collect::<Vec<_>>();
        block_pcs.sort();
        for start in block_pcs {
            let block = &self.contents[start];
            print!("{:#x}:", start);

            print!(" <- ");
            for pc in &block.prev {
                print!("{:#x} ", pc);
            }
            println!();

            for instr in &block.code {
                println!("  {}", instr);
            }

            print!("  -> ");
            for pc in &block.next {
                print!("{:#x} ", pc);
            }
            println!("\n");
        }
    }

    fn assert_valid(&self) -> &Self {
        for (pc, block) in &self.contents {
            for next in &block.next {
                assert!(
                    self.contents[next].prev.contains(&pc),
                    "Invalid CFG: {:#x} -> {:#x} but not the other way",
                    pc,
                    next
                );
            }
        }
        self
    }

    fn reduce(&mut self) {
        let mut progress = true;
        while progress {
            self.assert_valid();
            let keys: Vec<u16> = self.contents.keys().map(|k| *k).collect();
            progress = false;
            'step: for master_pc in keys {
                let next_count = self.contents.get(&master_pc).unwrap().next.len();
                if next_count == 1 {
                    let absorb_pc = self.contents.get(&master_pc).unwrap().next[0];
                    let absorb_block = self.contents.get(&absorb_pc).unwrap();
                    if absorb_block.prev.len() == 1 {
                        let absorb_removed = self.contents.remove(&absorb_pc).unwrap();
                        self.contents
                            .get_mut(&master_pc)
                            .unwrap()
                            .absorb_next(absorb_removed);

                        // Update prev pointers where necessary
                        for (_, block) in self.contents.iter_mut() {
                            for prev_pc in block.prev.iter_mut() {
                                if *prev_pc == absorb_pc {
                                    *prev_pc = master_pc;
                                }
                            }
                        }

                        progress = true;
                        break 'step;
                    }
                }
            }
        }
    }
}

pub fn analyze(prog: SrcProgram) {
    let mut flow_graph = Cfg::from_rom(prog.iter().map(|(_, m_instr)| match m_instr {
        Ok(instr) => Some(*instr),
        Err(_) => None,
    }));

    println!("Complete flow graph:");
    flow_graph.debug_print();
    flow_graph.assert_valid();

    flow_graph.reduce();

    println!("Reduced flow graph:");
    flow_graph.debug_print();
    flow_graph.assert_valid();
}

#[allow(dead_code)]
fn addr_to_idx(addr: Pc) -> Option<usize> {
    Some(addr.checked_sub(0x200)? as usize / 2)
}

#[allow(dead_code)]
fn idx_to_addr(idx: usize) -> Pc {
    200 * (idx * 2) as Pc
}

#[derive(Clone)]
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

    fn from_single(pc: Pc, instr: Instruction) -> Block {
        Block {
            code: vec![instr],
            prev: vec![],
            next: instr.next_pc(pc),
        }
    }

    fn absorb_next(&mut self, mut next_block: Block) {
        assert!(
            self.next.len() == 1,
            "Tried to absorb_next where there is more than 1 next already"
        );
        self.next = next_block.next;
        self.code.append(&mut next_block.code)
    }
}

trait AnalyzeInstruction {
    fn next_pc(&self, this_pc: Pc) -> Vec<Pc>;
    fn branches(&self) -> bool;
}

impl AnalyzeInstruction for Instruction {
    fn next_pc(&self, this_pc: Pc) -> Vec<Pc> {
        match *self {
            SKE(_, _) | SKPR(_) | SKUP(_) | SKNE(_, _) | SKRE(_, _) | SKRNE(_, _) => {
                vec![this_pc + 2, this_pc + 4]
            }
            JUMP(addr) => {
                vec![addr]
            }
            CALL(addr) => {
                vec![this_pc + 2, addr]
            }
            // TODO: What should be the next of an RTS?
            RTS => {
                vec![]
            }
            _ => vec![this_pc + 2],
        }
    }

    fn branches(&self) -> bool {
        self.next_pc(0).len() > 1
    }
}
