use crate::types::*;

use anyhow::Ok;

pub const RV32M_SET_R: [Instruction; 8] = [
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011 | 0b000 << FUNC3_POS | 0b0000001 << FUNC7_POS,
        name: "MUL",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);
            let rs1 = cpu.read_x_i32(instruction.rs1.value()) as i64;
            let rs2 = cpu.read_x_i32(instruction.rs2.value()) as i64;
            let (res, _) = rs1.overflowing_mul(rs2);
            cpu.write_x_i32(instruction.rd.value(), res as i32);

            cpu.debug_print(|| format!("MUL: rs1={}, rs2={}, res={}", rs1, rs2, res));

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011 | 0b001 << FUNC3_POS | 0b0000001 << FUNC7_POS,
        name: "MULH",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);
            let rs1 = cpu.read_x_i32(instruction.rs1.value()) as i64;
            let rs2 = cpu.read_x_i32(instruction.rs2.value()) as i64;
            let (res, _) = rs1.overflowing_mul(rs2);
            cpu.write_x_i32(instruction.rd.value(), (res >> 32) as i32);

            cpu.debug_print(|| format!("MUL: rs1={}, rs2={}, res={}", rs1, rs2, res));

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011 | 0b010 << FUNC3_POS | 0b0000001 << FUNC7_POS,
        name: "MULHSU",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1 = cpu.read_x_i32(instruction.rs1.value()) as i64;
            let rs2 = cpu.read_x_u32(instruction.rs2.value()) as i64;
            let (res, _) = rs1.overflowing_mul(rs2);
            cpu.write_x_u32(instruction.rd.value(), (res >> 32) as u32);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011 | 0b011 << FUNC3_POS | 0b0000001 << FUNC7_POS,
        name: "MULHU",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1 = cpu.read_x_u32(instruction.rs1.value()) as u64;
            let rs2 = cpu.read_x_u32(instruction.rs2.value()) as u64;
            let (res, _) = rs1.overflowing_mul(rs2);
            cpu.write_x_u32(instruction.rd.value(), (res >> 32) as u32);

            Ok(())
        },
    },
    Instruction {
        // TODO: Verify specifics of div calculations
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011 | 0b100 << FUNC3_POS | 0b0000001 << FUNC7_POS,
        name: "DIV",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1 = cpu.read_x_i32(instruction.rs1.value());
            let rs2 = cpu.read_x_i32(instruction.rs2.value());

            if rs2 == 0 {
                cpu.write_x_i32(instruction.rd.value(), -1);
            } else if rs1 == i32::MIN && rs2 == -1 {
                cpu.write_x_i32(instruction.rd.value(), i32::MIN);
            } else {
                cpu.write_x_i32(instruction.rd.value(), rs1 / rs2);
            }
            Ok(())
        },
    },
    Instruction {
        // TODO: Verify specifics of div calculations
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011 | 0b101 << FUNC3_POS | 0b0000001 << FUNC7_POS,
        name: "DIVU",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1 = cpu.read_x_u32(instruction.rs1.value());
            let rs2 = cpu.read_x_u32(instruction.rs2.value());

            if rs2 == 0 {
                cpu.write_x_u32(instruction.rd.value(), u32::MAX);
            } else {
                cpu.write_x_u32(instruction.rd.value(), rs1 / rs2);
            }

            Ok(())
        },
    },
    Instruction {
        // TODO: Verify specifics of div calculations
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011 | 0b110 << FUNC3_POS | 0b0000001 << FUNC7_POS,
        name: "REM",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1 = cpu.read_x_i32(instruction.rs1.value());
            let rs2 = cpu.read_x_i32(instruction.rs2.value());

            if rs2 == 0 {
                cpu.write_x_i32(instruction.rd.value(), rs1);
            } else {
                cpu.write_x_i32(instruction.rd.value(), rs1 % rs2);
            }
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011 | 0b111 << FUNC3_POS | 0b0000001 << FUNC7_POS,
        name: "REMU",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1 = cpu.read_x_u32(instruction.rs1.value());
            let rs2 = cpu.read_x_u32(instruction.rs2.value());

            if rs2 == 0 {
                cpu.write_x_u32(instruction.rd.value(), rs1);
            } else {
                cpu.write_x_u32(instruction.rd.value(), rs1 % rs2);
            }
            Ok(())
        },
    },
];
