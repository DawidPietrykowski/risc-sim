use crate::isa::types::*;

use anyhow::Ok;

pub const RV32I_SET_FLOAT: [Instruction; 1] = [Instruction {
    mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
    bits: 0b0110011 | 0b000 << FUNC3_POS | 0b0000001 << FUNC7_POS,
    name: "MUL",
    instruction_type: InstructionType::R,
    operation: |cpu, word| {
        let instruction = parse_instruction_r(word);
        let rs1 = cpu.read_x_u32(instruction.rs1.value())? as u64;
        let rs2 = cpu.read_x_u32(instruction.rs2.value())? as u64;
        let res = rs1 * rs2;
        cpu.write_x_u32(instruction.rd.value(), res as u32)?;
        Ok(())
    },
}];
