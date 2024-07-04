use crate::isa::types::*;

pub const RV32I_SET_E: [Instruction; 2] = [
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC12_MASK,
        bits: 0b0 << FUNC12_POS | 0b1110011,
        name: "ECALL",
        instruction_type: InstructionType::I,
        operation: |_cpu, _word| {
            todo!();
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC12_MASK,
        bits: 0b1 << FUNC12_POS | 0b1110011,
        name: "EBREAK",
        instruction_type: InstructionType::I,
        operation: |_cpu, _word| {
            todo!();
        },
    },
];
