use crate::isa::cpu::{Cpu, Operation};
use crate::isa::types::*;

use anyhow::{Ok, Result};

pub struct JAL {
    instruction: UJInstructionData,
}

impl Operation<UJInstructionData> for JAL {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let extended_offset = self.instruction.imm.as_i32();
        let moved_pc = cpu.read_pc_u32().wrapping_add_signed(extended_offset);
        cpu.write_pc_u32(moved_pc);

        cpu.set_skip_pc_increment_flag();

        Ok(())
    }

    fn new(instruction: UJInstructionData) -> Self {
        JAL {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &UJInstructionData {
        &self.instruction
    }
}