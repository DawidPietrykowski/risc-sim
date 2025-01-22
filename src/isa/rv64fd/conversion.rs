use crate::types::*;

use anyhow::Ok;

pub const RV64F_SET_CONVERSION: [Instruction; 16] = [
    Instruction {
        mask: OPCODE_MASK | RS2_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b00000 << RS2_POS | 0b1101000 << FUNC7_POS,
        name: "FCVT.S.W",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let rs1_val = cpu.read_x_i32(instruction.rs1.value());

            cpu.write_f32(instruction.rd.value(), rs1_val as f32);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | RS2_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b00000 << RS2_POS | 0b1100000 << FUNC7_POS,
        name: "FCVT.W.S",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let rs1_val = cpu.read_f32(instruction.rs1.value());

            cpu.write_x_i32(instruction.rd.value(), rs1_val as i32);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | RS2_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b00000 << RS2_POS | 0b1100001 << FUNC7_POS,
        name: "FCVT.W.D",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let rs1_val = cpu.read_f64(instruction.rs1.value());

            cpu.write_x_i32(instruction.rd.value(), rs1_val as i32);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | RS2_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b00001 << RS2_POS | 0b1100000 << FUNC7_POS,
        name: "FCVT.WU.S",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let rs1_val = cpu.read_f32(instruction.rs1.value());

            cpu.write_x_u32(instruction.rd.value(), rs1_val as u32);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | RS2_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b00001 << RS2_POS | 0b1100001 << FUNC7_POS,
        name: "FCVT.WU.D",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let rs1_val = cpu.read_f64(instruction.rs1.value());

            cpu.write_x_u32(instruction.rd.value(), rs1_val as u32);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | RS2_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b00010 << RS2_POS | 0b1100000 << FUNC7_POS,
        name: "FCVT.L.S",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let rs1_val = cpu.read_f32(instruction.rs1.value());

            cpu.write_x_i64(instruction.rd.value(), rs1_val as i64);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | RS2_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b00011 << RS2_POS | 0b1100000 << FUNC7_POS,
        name: "FCVT.LU.S",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let rs1_val = cpu.read_f32(instruction.rs1.value());

            cpu.write_x_u64(instruction.rd.value(), rs1_val as u64);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | RS2_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b00010 << RS2_POS | 0b1100001 << FUNC7_POS,
        name: "FCVT.L.D",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let rs1_val = cpu.read_f64(instruction.rs1.value());

            cpu.write_x_i64(instruction.rd.value(), rs1_val as i64);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | RS2_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b00011 << RS2_POS | 0b1100001 << FUNC7_POS,
        name: "FCVT.LU.D",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let rs1_val = cpu.read_f64(instruction.rs1.value());

            cpu.write_x_u64(instruction.rd.value(), rs1_val as u64);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | RS2_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b00001 << RS2_POS | 0b0100000 << FUNC7_POS,
        name: "FCVT.S.D",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let rs1_val = cpu.read_f64(instruction.rs1.value());

            cpu.write_f32(instruction.rd.value(), rs1_val as f32);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | RS2_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b00010 << RS2_POS | 0b1101000 << FUNC7_POS,
        name: "FCVT.S.L",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let rs1_val = cpu.read_x_i64(instruction.rs1.value());

            cpu.write_f32(instruction.rd.value(), rs1_val as f32);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | RS2_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b00000 << RS2_POS | 0b0100001 << FUNC7_POS,
        name: "FCVT.D.S",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let rs1_val = cpu.read_f32(instruction.rs1.value());

            cpu.write_f64(instruction.rd.value(), rs1_val as f64);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | RS2_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b00000 << RS2_POS | 0b1101001 << FUNC7_POS,
        name: "FCVT.D.W",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let rs1_val = cpu.read_x_i32(instruction.rs1.value());

            cpu.write_f64(instruction.rd.value(), rs1_val as f64);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | RS2_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b00001 << RS2_POS | 0b1101001 << FUNC7_POS,
        name: "FCVT.D.WU",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let rs1_val = cpu.read_x_u32(instruction.rs1.value());

            cpu.write_f64(instruction.rd.value(), rs1_val as f64);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | RS2_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b00010 << RS2_POS | 0b1101001 << FUNC7_POS,
        name: "FCVT.D.L",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let rs1_val = cpu.read_x_i64(instruction.rs1.value());

            cpu.write_f64(instruction.rd.value(), rs1_val as f64);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | RS2_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b00011 << RS2_POS | 0b1101001 << FUNC7_POS,
        name: "FCVT.D.LU",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let rs1_val = cpu.read_x_u64(instruction.rs1.value());

            cpu.write_f64(instruction.rd.value(), rs1_val as f64);

            Ok(())
        },
    },
];
