use crate::{
    types::*,
    utils::binary_utils::{f32_to_u32, f64_to_u64, u32_to_f32, u64_to_f64},
};

use anyhow::Ok;

pub const RV64F_SET_LS: [Instruction; 4] = [
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0000111 | 0b010 << FUNC3_POS,
        name: "FLW",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let extended_offset = instruction.imm.as_i64();
            let moved_addr = cpu
                .read_x_u64(instruction.rs1.value())?
                .wrapping_add_signed(extended_offset);

            cpu.debug_print(|| {
                format!(
                    "FLW: r{}({:#x}) = mem[r{:#x} + {:#x}] (addr: {:#x})",
                    instruction.rd.value(),
                    instruction.rs1.value(),
                    cpu.read_x_u64(instruction.rs1.value()).unwrap(),
                    extended_offset,
                    moved_addr
                )
            });

            let read_value = u32_to_f32(cpu.read_mem_u32(moved_addr)?);

            cpu.debug_print(|| format!("FLW: {:x?}", read_value));

            cpu.write_f32(instruction.rd.value(), read_value)?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0000111 | 0b011 << FUNC3_POS,
        name: "FLD",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let extended_offset = instruction.imm.as_i64();
            let moved_addr = cpu
                .read_x_u64(instruction.rs1.value())?
                .wrapping_add_signed(extended_offset);

            let read_value = u64_to_f64(cpu.read_mem_u64(moved_addr)?);

            cpu.debug_print(|| format!("FLD: {:#x?}", read_value));

            cpu.write_f64(instruction.rd.value(), read_value)?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0100111 | 0b010 << FUNC3_POS,
        name: "FSW",
        instruction_type: InstructionType::S,
        operation: |cpu, word| {
            let instruction = parse_instruction_s(word);

            let extended_offset = instruction.imm.as_i64();
            let rs1 = cpu.read_x_u64(instruction.rs1.value())?;
            let moved_addr = cpu
                .read_x_u64(instruction.rs1.value())?
                .wrapping_add_signed(extended_offset);
            let read_value = cpu.read_f32(instruction.rs2.value())?;

            cpu.debug_print(|| {
                format!(
                    "FSW: {:#x} = {:#x?} (addr: {:#x} + {}) word: {:#x}",
                    moved_addr, read_value, rs1, extended_offset, word.0
                )
            });

            cpu.write_mem_u32(moved_addr, f32_to_u32(read_value))?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b0100111 | 0b011 << FUNC3_POS,
        name: "FSD",
        instruction_type: InstructionType::S,
        operation: |cpu, word| {
            let instruction = parse_instruction_s(word);

            let extended_offset = instruction.imm.as_i64();
            let rs1 = cpu.read_x_u64(instruction.rs1.value())?;
            let moved_addr = cpu
                .read_x_u64(instruction.rs1.value())?
                .wrapping_add_signed(extended_offset);
            let read_value = cpu.read_f64(instruction.rs2.value())?;

            cpu.debug_print(|| {
                format!(
                    "SD: {:#x} = {:#x?} (addr: {:#x} + {}) dword: {:#x}",
                    moved_addr, read_value, rs1, extended_offset, word.0
                )
            });

            cpu.write_mem_u64(moved_addr, f64_to_u64(read_value))?;

            Ok(())
        },
    },
];
