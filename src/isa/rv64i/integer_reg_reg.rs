use crate::{types::*, utils::binary_utils::sign_extend_32bit_to_64bit};

use anyhow::Ok;

pub const RV64I_SET_R: [Instruction; 15] = [
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011,
        name: "ADD",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);
            let rs1 = cpu.read_x_i64(instruction.rs1.value());
            let rs2 = cpu.read_x_i64(instruction.rs2.value());
            let (res, _) = rs1.overflowing_add(rs2);
            cpu.write_x_i64(instruction.rd.value(), res);

            cpu.debug_print(|| format!("ADD: rs1={}, rs2={}, res={}", rs1, rs2, res));

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011 | 0b0100000 << FUNC7_POS,
        name: "SUB",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);
            let rs1 = cpu.read_x_i64(instruction.rs1.value());
            let rs2 = cpu.read_x_i64(instruction.rs2.value());
            let (res, _) = rs1.overflowing_sub(rs2);
            cpu.write_x_i64(instruction.rd.value(), res);
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0111011,
        name: "ADDW",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);
            let rs1 = cpu.read_x_i64(instruction.rs1.value()) as i32;
            let rs2 = cpu.read_x_i64(instruction.rs2.value()) as i32;
            let (res, _) = rs1.overflowing_add(rs2);
            cpu.write_x_i64(instruction.rd.value(), res as i64);

            cpu.debug_print(|| format!("ADD: rs1={}, rs2={}, res={}", rs1, rs2, res));

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0111011 | 0b0100000 << FUNC7_POS,
        name: "SUBW",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);
            let rs1 = cpu.read_x_i64(instruction.rs1.value()) as i32;
            let rs2 = cpu.read_x_i64(instruction.rs2.value()) as i32;
            let (res, _) = rs1.overflowing_sub(rs2);
            cpu.write_x_i64(instruction.rd.value(), res as i64);
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011 | 0b001 << FUNC3_POS | 0b0000000 << FUNC7_POS,
        name: "SLL",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);
            let shamt = (cpu.read_x_u64(instruction.rs2.value()) & (U6_MASK as u64)) as u64;
            let res: u64 = cpu.read_x_u64(instruction.rs1.value()) << shamt;
            cpu.write_x_u64(instruction.rd.value(), res);
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0111011 | 0b001 << FUNC3_POS | 0b0000000 << FUNC7_POS,
        name: "SLLW",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);
            let shamt = (cpu.read_x_u64(instruction.rs2.value()) & (U5_MASK as u64)) as u32;
            let res: u32 = (cpu.read_x_u64(instruction.rs1.value()) as u32) << shamt;
            cpu.write_x_i64(instruction.rd.value(), sign_extend_32bit_to_64bit(res));
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011 | 0b010 << FUNC3_POS | 0b0000000 << FUNC7_POS,
        name: "SLT",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);
            let rs1 = cpu.read_x_i64(instruction.rs1.value());
            let rs2 = cpu.read_x_i64(instruction.rs2.value());
            cpu.write_x_i64(instruction.rd.value(), if rs1 < rs2 { 1 } else { 0 });
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011 | 0b011 << FUNC3_POS | 0b0000000 << FUNC7_POS,
        name: "SLTU",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);
            let rs1 = cpu.read_x_u64(instruction.rs1.value());
            let rs2 = cpu.read_x_u64(instruction.rs2.value());
            cpu.write_x_i64(instruction.rd.value(), if rs1 < rs2 { 1 } else { 0 });
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011 | 0b100 << FUNC3_POS | 0b0000000 << FUNC7_POS,
        name: "XOR",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);
            let rs1 = cpu.read_x_u64(instruction.rs1.value());
            let rs2 = cpu.read_x_u64(instruction.rs2.value());
            cpu.write_x_u64(instruction.rd.value(), rs1 ^ rs2);
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011 | 0b101 << FUNC3_POS | 0b0000000 << FUNC7_POS,
        name: "SRL",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);
            let shamt = (cpu.read_x_u64(instruction.rs2.value()) & 0b111111) as u64;
            let res: u64 = cpu.read_x_u64(instruction.rs1.value()) >> shamt;
            cpu.write_x_u64(instruction.rd.value(), res);
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0111011 | 0b101 << FUNC3_POS | 0b0000000 << FUNC7_POS,
        name: "SRLW",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);
            let shamt = (cpu.read_x_u64(instruction.rs2.value()) & 0b11111) as u32;
            let res: u32 = (cpu.read_x_u64(instruction.rs1.value()) as u32) >> shamt;
            cpu.write_x_i64(instruction.rd.value(), sign_extend_32bit_to_64bit(res));
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011 | 0b101 << FUNC3_POS | 0b0100000 << FUNC7_POS,
        name: "SRA",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);
            let shamt = (cpu.read_x_u64(instruction.rs2.value()) & 0b111111) as u64;
            let res: i64 = cpu.read_x_i64(instruction.rs1.value()) >> shamt;
            cpu.write_x_i64(instruction.rd.value(), res);
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0111011 | 0b101 << FUNC3_POS | 0b0100000 << FUNC7_POS,
        name: "SRAW",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);
            let shamt = (cpu.read_x_u64(instruction.rs2.value()) & 0b11111) as u64;
            let res: i32 = (cpu.read_x_i64(instruction.rs1.value()) as i32) >> shamt;
            cpu.write_x_i64(instruction.rd.value(), res as i64);
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011 | 0b110 << FUNC3_POS | 0b0000000 << FUNC7_POS,
        name: "OR",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);
            let rs1 = cpu.read_x_u64(instruction.rs1.value());
            let rs2 = cpu.read_x_u64(instruction.rs2.value());
            cpu.write_x_u64(instruction.rd.value(), rs1 | rs2);
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011 | 0b111 << FUNC3_POS | 0b0000000 << FUNC7_POS,
        name: "AND",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);
            let rs1 = cpu.read_x_u64(instruction.rs1.value());
            let rs2 = cpu.read_x_u64(instruction.rs2.value());
            cpu.write_x_u64(instruction.rd.value(), rs1 & rs2);
            Ok(())
        },
    },
];
