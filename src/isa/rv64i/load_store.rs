use crate::{
    types::*,
    utils::binary_utils::{sign_extend_32bit_to_64bit, u16_to_i16, u8_to_i8},
};

use anyhow::Ok;

pub const RV64I_SET_LS: [Instruction; 11] = [
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0000011 | 0b000 << FUNC3_POS,
        name: "LB",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let extended_offset = instruction.imm.as_i64();
            let moved_addr = cpu
                .read_x_u64(instruction.rs1.value())
                .wrapping_add_signed(extended_offset);
            let read_value = u8_to_i8(cpu.read_mem_u8(moved_addr)?) as i32;

            cpu.write_x_u64(instruction.rd.value(), read_value as u64);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0000011 | 0b001 << FUNC3_POS,
        name: "LH",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let extended_offset = instruction.imm.as_i64();
            let moved_addr = cpu
                .read_x_u64(instruction.rs1.value())
                .wrapping_add_signed(extended_offset);
            let read_value = u16_to_i16(cpu.read_mem_u16(moved_addr)?) as i32;

            cpu.write_x_u64(instruction.rd.value(), read_value as u64);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0000011 | 0b010 << FUNC3_POS,
        name: "LW",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let extended_offset = instruction.imm.as_i64();
            let moved_addr = cpu
                .read_x_u64(instruction.rs1.value())
                .wrapping_add_signed(extended_offset);

            let read_value = cpu.read_mem_u32(moved_addr)?;

            cpu.write_x_i64(
                instruction.rd.value(),
                sign_extend_32bit_to_64bit(read_value),
            );

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0000011 | 0b011 << FUNC3_POS,
        name: "LD",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let extended_offset = instruction.imm.as_i64();
            let moved_addr = cpu
                .read_x_u64(instruction.rs1.value())
                .wrapping_add_signed(extended_offset);

            let read_value = cpu.read_mem_u64(moved_addr)?;

            cpu.write_x_u64(instruction.rd.value(), read_value);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0000011 | 0b110 << FUNC3_POS,
        name: "LWU",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let extended_offset = instruction.imm.as_i64();
            let moved_addr = cpu
                .read_x_u64(instruction.rs1.value())
                .wrapping_add_signed(extended_offset);

            let read_value = cpu.read_mem_u32(moved_addr)?;

            cpu.write_x_u64(instruction.rd.value(), read_value as u64);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0000011 | 0b100 << FUNC3_POS,
        name: "LBU",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let extended_offset = instruction.imm.as_i64();
            let moved_addr = cpu
                .read_x_u64(instruction.rs1.value())
                .wrapping_add_signed(extended_offset);
            let read_value = cpu.read_mem_u8(moved_addr)?;

            cpu.write_x_u64(instruction.rd.value(), read_value as u64);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0000011 | 0b101 << FUNC3_POS,
        name: "LHU",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let extended_offset = instruction.imm.as_i64();
            let moved_addr = cpu
                .read_x_u64(instruction.rs1.value())
                .wrapping_add_signed(extended_offset);
            let read_value = cpu.read_mem_u16(moved_addr)?;

            cpu.write_x_u64(instruction.rd.value(), read_value as u64);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0100011 | 0b010 << FUNC3_POS,
        name: "SW",
        instruction_type: InstructionType::S,
        operation: |cpu, word| {
            let instruction = parse_instruction_s(word);

            let extended_offset = instruction.imm.as_i64();
            let moved_addr = cpu
                .read_x_u64(instruction.rs1.value())
                .wrapping_add_signed(extended_offset);
            let read_value = cpu.read_x_u64(instruction.rs2.value());

            cpu.write_mem_u32(moved_addr, read_value as u32)?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0100011 | 0b011 << FUNC3_POS,
        name: "SD",
        instruction_type: InstructionType::S,
        operation: |cpu, word| {
            let instruction = parse_instruction_s(word);

            let extended_offset = instruction.imm.as_i64();
            let rs1 = cpu.read_x_u64(instruction.rs1.value());
            let moved_addr = rs1.wrapping_add_signed(extended_offset);
            let read_value = cpu.read_x_u64(instruction.rs2.value());

            cpu.write_mem_u64(moved_addr, read_value)?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0100011 | 0b001 << FUNC3_POS,
        name: "SH",
        instruction_type: InstructionType::S,
        operation: |cpu, word| {
            let instruction = parse_instruction_s(word);

            let extended_offset = instruction.imm.as_i64();
            let moved_addr = cpu
                .read_x_u64(instruction.rs1.value())
                .wrapping_add_signed(extended_offset);
            let read_value = cpu.read_x_u64(instruction.rs2.value());

            cpu.write_mem_u16(moved_addr, read_value as u16)?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0100011 | 0b000 << FUNC3_POS,
        name: "SB",
        instruction_type: InstructionType::S,
        operation: |cpu, word| {
            let instruction = parse_instruction_s(word);

            let extended_offset = instruction.imm.as_i64();
            let moved_addr = cpu
                .read_x_u64(instruction.rs1.value())
                .wrapping_add_signed(extended_offset);
            let read_value = cpu.read_x_u64(instruction.rs2.value());

            cpu.write_mem_u8(moved_addr, read_value as u8)?;

            Ok(())
        },
    },
];
