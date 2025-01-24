use crate::{types::*, utils::binary_utils::sign_extend_32bit_to_64bit};

use anyhow::Ok;

pub const RV64M_SET_R: [Instruction; 13] = [
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011 | 0b000 << FUNC3_POS | 0b0000001 << FUNC7_POS,
        name: "MUL",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);
            let rs1 = cpu.read_x_i64(instruction.rs1.value()) as i64;
            let rs2 = cpu.read_x_i64(instruction.rs2.value()) as i64;
            let (res, _) = rs1.overflowing_mul(rs2);
            cpu.write_x_i64(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0111011 | 0b000 << FUNC3_POS | 0b0000001 << FUNC7_POS,
        name: "MULW",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);
            let rs1 = cpu.read_x_i64(instruction.rs1.value()) as i32;
            let rs2 = cpu.read_x_i64(instruction.rs2.value()) as i32;
            let (res, _) = rs1.overflowing_mul(rs2);
            cpu.write_x_i64(instruction.rd.value(), res as i64);

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
            let rs1 = cpu.read_x_i64(instruction.rs1.value()) as i128;
            let rs2 = cpu.read_x_i64(instruction.rs2.value()) as i128;
            let (res, _) = rs1.overflowing_mul(rs2);
            cpu.write_x_i64(instruction.rd.value(), (res >> 64) as i64);

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

            let rs1 = cpu.read_x_i64(instruction.rs1.value()) as i128;
            let rs2 = cpu.read_x_u64(instruction.rs2.value()) as i128;
            let (res, _) = rs1.overflowing_mul(rs2);
            cpu.write_x_u64(instruction.rd.value(), (res >> 64) as u64);

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

            let rs1 = cpu.read_x_u64(instruction.rs1.value()) as u128;
            let rs2 = cpu.read_x_u64(instruction.rs2.value()) as u128;
            let (res, _) = rs1.overflowing_mul(rs2);
            cpu.write_x_u64(instruction.rd.value(), (res >> 64) as u64);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011 | 0b100 << FUNC3_POS | 0b0000001 << FUNC7_POS,
        name: "DIV",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1 = cpu.read_x_i64(instruction.rs1.value());
            let rs2 = cpu.read_x_i64(instruction.rs2.value());

            if rs2 == 0 {
                cpu.write_x_i64(instruction.rd.value(), -1);
            } else if rs1 == i64::MIN && rs2 == -1 {
                cpu.write_x_i64(instruction.rd.value(), i64::MIN);
            } else {
                cpu.write_x_i64(instruction.rd.value(), rs1 / rs2);
            }
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0111011 | 0b100 << FUNC3_POS | 0b0000001 << FUNC7_POS,
        name: "DIVW",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1 = cpu.read_x_i64(instruction.rs1.value()) as i32;
            let rs2 = cpu.read_x_i64(instruction.rs2.value()) as i32;

            if rs2 == 0 {
                cpu.write_x_i64(instruction.rd.value(), -1);
            } else if rs1 == i32::MIN && rs2 == -1 {
                cpu.write_x_i64(instruction.rd.value(), i64::MIN);
            } else {
                cpu.write_x_i64(instruction.rd.value(), (rs1 / rs2) as i64);
            }
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011 | 0b101 << FUNC3_POS | 0b0000001 << FUNC7_POS,
        name: "DIVU",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1 = cpu.read_x_u64(instruction.rs1.value());
            let rs2 = cpu.read_x_u64(instruction.rs2.value());

            if rs2 == 0 {
                cpu.write_x_u64(instruction.rd.value(), u64::MAX);
            } else {
                cpu.write_x_u64(instruction.rd.value(), rs1 / rs2);
            }

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0111011 | 0b101 << FUNC3_POS | 0b0000001 << FUNC7_POS,
        name: "DIVUW",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1 = cpu.read_x_u64(instruction.rs1.value()) as u32;
            let rs2 = cpu.read_x_u64(instruction.rs2.value()) as u32;

            if rs2 == 0 {
                cpu.write_x_u64(instruction.rd.value(), u64::MAX);
            } else {
                cpu.write_x_i64(
                    instruction.rd.value(),
                    sign_extend_32bit_to_64bit(rs1 / rs2),
                );
            }

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0110011 | 0b110 << FUNC3_POS | 0b0000001 << FUNC7_POS,
        name: "REM",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1 = cpu.read_x_i64(instruction.rs1.value());
            let rs2 = cpu.read_x_i64(instruction.rs2.value());

            if rs2 == 0 {
                cpu.write_x_i64(instruction.rd.value(), rs1);
            } else {
                cpu.write_x_i64(instruction.rd.value(), rs1 % rs2);
            }
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0111011 | 0b110 << FUNC3_POS | 0b0000001 << FUNC7_POS,
        name: "REMW",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1 = cpu.read_x_i64(instruction.rs1.value()) as i32;
            let rs2 = cpu.read_x_i64(instruction.rs2.value()) as i32;

            if rs2 == 0 {
                cpu.write_x_i64(instruction.rd.value(), rs1 as i64);
            } else {
                cpu.write_x_i64(instruction.rd.value(), (rs1 % rs2) as i64);
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

            let rs1 = cpu.read_x_u64(instruction.rs1.value());
            let rs2 = cpu.read_x_u64(instruction.rs2.value());

            if rs2 == 0 {
                cpu.write_x_u64(instruction.rd.value(), rs1);
            } else {
                cpu.write_x_u64(instruction.rd.value(), rs1 % rs2);
            }
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0111011 | 0b111 << FUNC3_POS | 0b0000001 << FUNC7_POS,
        name: "REMUW",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1 = cpu.read_x_u64(instruction.rs1.value()) as u32;
            let rs2 = cpu.read_x_u64(instruction.rs2.value()) as u32;

            if rs2 == 0 {
                cpu.write_x_i64(instruction.rd.value(), sign_extend_32bit_to_64bit(rs1));
            } else {
                cpu.write_x_i64(
                    instruction.rd.value(),
                    sign_extend_32bit_to_64bit(rs1 % rs2),
                );
            }
            Ok(())
        },
    },
];
