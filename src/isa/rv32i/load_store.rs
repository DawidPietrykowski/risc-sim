use crate::{
    isa::types::*,
    utils::binary_utils::{u16_to_i16, u8_to_i8},
};

use anyhow::Ok;

pub const RV32I_SET_LS: [Instruction; 8] = [
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0000011 | 0b000 << FUNC3_POS,
        name: "LB",
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let extended_offset = instruction.imm.as_i32();
            let moved_addr = cpu
                .read_x_u32(instruction.rs1.value())?
                .wrapping_add_signed(extended_offset);
            let read_value = u8_to_i8(cpu.read_mem_u8(moved_addr)?) as i32;

            cpu.write_x_u32(instruction.rd.value(), read_value as u32)?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0000011 | 0b001 << FUNC3_POS,
        name: "LH",
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let extended_offset = instruction.imm.as_i32();
            let moved_addr = cpu
                .read_x_u32(instruction.rs1.value())?
                .wrapping_add_signed(extended_offset);
            let read_value = u16_to_i16(cpu.read_mem_u16(moved_addr)?) as i32;

            cpu.write_x_u32(instruction.rd.value(), read_value as u32)?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0000011 | 0b010 << FUNC3_POS,
        name: "LW",
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
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0000011 | 0b100 << FUNC3_POS,
        name: "LBU",
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let extended_offset = instruction.imm.as_i32();
            let moved_addr = cpu
                .read_x_u32(instruction.rs1.value())?
                .wrapping_add_signed(extended_offset);
            let read_value = cpu.read_mem_u8(moved_addr)?;

            cpu.write_x_u32(instruction.rd.value(), read_value as u32)?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0000011 | 0b101 << FUNC3_POS,
        name: "LHU",
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let extended_offset = instruction.imm.as_i32();
            let moved_addr = cpu
                .read_x_u32(instruction.rs1.value())?
                .wrapping_add_signed(extended_offset);
            let read_value = cpu.read_mem_u16(moved_addr)?;

            cpu.write_x_u32(instruction.rd.value(), read_value as u32)?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0100011 | 0b010 << FUNC3_POS,
        name: "SW",
        operation: |cpu, word| {
            let instruction = parse_instruction_s(word);

            let extended_offset = instruction.imm.as_i32();
            let moved_addr = cpu
                .read_x_u32(instruction.rs1.value())?
                .wrapping_add_signed(extended_offset);
            let read_value = cpu.read_x_u32(instruction.rs2.value())?;

            cpu.write_mem_u32(moved_addr, read_value)?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0100011 | 0b001 << FUNC3_POS,
        name: "SH",
        operation: |cpu, word| {
            let instruction = parse_instruction_s(word);

            let extended_offset = instruction.imm.as_i32();
            let moved_addr = cpu
                .read_x_u32(instruction.rs1.value())?
                .wrapping_add_signed(extended_offset);
            let read_value = cpu.read_x_u32(instruction.rs2.value())?;

            cpu.write_mem_u16(moved_addr, read_value as u16)?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0100011 | 0b000 << FUNC3_POS,
        name: "SB",
        operation: |cpu, word| {
            let instruction = parse_instruction_s(word);

            let extended_offset = instruction.imm.as_i32();
            let moved_addr = cpu
                .read_x_u32(instruction.rs1.value())?
                .wrapping_add_signed(extended_offset);
            let read_value = cpu.read_x_u32(instruction.rs2.value())?;

            cpu.write_mem_u8(moved_addr, read_value as u8)?;

            Ok(())
        },
    },
];