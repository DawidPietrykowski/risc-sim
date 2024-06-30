use crate::isa::types::*;

use anyhow::Ok;

pub const RV32I_SET_LS: [Instruction; 2] = [
    Instruction {
        mask: OPCODE_MASK,
        bits: 0b0000011,
        name: "LOAD",
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let extended_offset = instruction.imm.as_i32();
            let moved_addr = cpu
                .read_x_u32(instruction.rs1.value())?
                .wrapping_add_signed(extended_offset);
            let read_value = cpu.read_mem_u32(moved_addr)?;

            cpu.write_x_u32(instruction.rd.value(), read_value)?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK,
        bits: 0b0100011,
        name: "STORE",
        operation: |cpu, word| {
            let instruction = parse_instruction_s(word);

            let extended_offset = instruction.imm.as_i32();
            let moved_addr = cpu
                .read_x_u32(instruction.rs1.value())?
                .wrapping_add_signed(extended_offset);
            let read_value = cpu.read_x_u32(instruction.rs2.value())?;

            cpu.write_mem_u32(moved_addr, read_value);

            Ok(())
        },
    },
];
