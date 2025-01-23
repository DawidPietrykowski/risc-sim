use crate::types::*;

use anyhow::Ok;

pub const RV32_ZIFENCEI_SET: [Instruction; 1] = [Instruction {
    mask: OPCODE_MASK | FUNC3_MASK,
    bits: 0b0001111 | 0b001 << FUNC3_POS,
    name: "FENCE.I",
    instruction_type: InstructionType::R,
    operation: |_cpu, _word| Ok(()),
}];
