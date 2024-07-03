use crate::isa::types::*;
use crate::utils::binary_utils::*;

use anyhow::Ok;

pub const RV32I_SET_E: [Instruction; 2] = [
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC12_MASK,
        bits: 0b0 << FUNC12_POS | 0b1110011,
        name: "ECALL",
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            todo!();
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC12_MASK,
        bits: 0b1 << FUNC12_POS | 0b1110011,
        name: "EBREAK",
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            todo!();
            Ok(())
        },
    },
];