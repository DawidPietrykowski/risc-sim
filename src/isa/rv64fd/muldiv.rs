use crate::types::*;

use anyhow::Ok;

pub const RV64F_SET_MULDIV: [Instruction; 22] = [
    Instruction {
        mask: OPCODE_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b0001000 << FUNC7_POS,
        name: "FMUL.S",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f32(instruction.rs1.value());
            let rs2_val = cpu.read_f32(instruction.rs2.value());
            let res = rs1_val * rs2_val;

            cpu.write_f32(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b0001001 << FUNC7_POS,
        name: "FMUL.D",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f64(instruction.rs1.value());
            let rs2_val = cpu.read_f64(instruction.rs2.value());
            let res = rs1_val * rs2_val;

            cpu.write_f64(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b0001100 << FUNC7_POS,
        name: "FDIV.S",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f32(instruction.rs1.value());
            let rs2_val = cpu.read_f32(instruction.rs2.value());
            let res = rs1_val / rs2_val;

            cpu.write_f32(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b0001101 << FUNC7_POS,
        name: "FDIV.D",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f64(instruction.rs1.value());
            let rs2_val = cpu.read_f64(instruction.rs2.value());
            let res = rs1_val / rs2_val;

            cpu.write_f64(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b0000000 << FUNC7_POS,
        name: "FADD.S",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f32(instruction.rs1.value());
            let rs2_val = cpu.read_f32(instruction.rs2.value());
            let res = rs1_val + rs2_val;

            cpu.write_f32(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b0000100 << FUNC7_POS,
        name: "FSUB.S",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f32(instruction.rs1.value());
            let rs2_val = cpu.read_f32(instruction.rs2.value());
            let res = rs1_val - rs2_val;

            cpu.write_f32(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b0000001 << FUNC7_POS,
        name: "FADD.D",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f64(instruction.rs1.value());
            let rs2_val = cpu.read_f64(instruction.rs2.value());
            let res = rs1_val + rs2_val;

            cpu.write_f64(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b0000101 << FUNC7_POS,
        name: "FSUB.D",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f64(instruction.rs1.value());
            let rs2_val = cpu.read_f64(instruction.rs2.value());
            let res = rs1_val - rs2_val;

            cpu.write_f64(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC2_MASK,
        bits: 0b1000011 | 0b00 << FUNC2_POS,
        name: "FMADD.S",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f32(instruction.rs1.value());
            let rs2_val = cpu.read_f32(instruction.rs2.value());
            let rs3_val = cpu.read_f32(instruction.func7.value() >> 2);

            let res = (rs1_val * rs2_val) + rs3_val;

            cpu.write_f32(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC2_MASK,
        bits: 0b1000111 | 0b00 << FUNC2_POS,
        name: "FMSUB.S",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f32(instruction.rs1.value());
            let rs2_val = cpu.read_f32(instruction.rs2.value());
            let rs3_val = cpu.read_f32(instruction.func7.value() >> 2);

            let res = (rs1_val * rs2_val) - rs3_val;

            cpu.write_f32(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC2_MASK,
        bits: 0b1001011 | 0b00 << FUNC2_POS,
        name: "FNMSUB.S",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f32(instruction.rs1.value());
            let rs2_val = cpu.read_f32(instruction.rs2.value());
            let rs3_val = cpu.read_f32(instruction.func7.value() >> 2);

            let res = -(rs1_val * rs2_val) - rs3_val;

            cpu.write_f32(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC2_MASK,
        bits: 0b1001111 | 0b00 << FUNC2_POS,
        name: "FNMADD.S",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f32(instruction.rs1.value());
            let rs2_val = cpu.read_f32(instruction.rs2.value());
            let rs3_val = cpu.read_f32(instruction.func7.value() >> 2);

            let res = -(rs1_val * rs2_val) + rs3_val;

            cpu.write_f32(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC2_MASK,
        bits: 0b1000011 | 0b01 << FUNC2_POS,
        name: "FMADD.D",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f64(instruction.rs1.value());
            let rs2_val = cpu.read_f64(instruction.rs2.value());
            let rs3_val = cpu.read_f64(instruction.func7.value() >> 2);

            let res = (rs1_val * rs2_val) + rs3_val;

            cpu.write_f64(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC2_MASK,
        bits: 0b1000111 | 0b01 << FUNC2_POS,
        name: "FMSUB.S",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f64(instruction.rs1.value());
            let rs2_val = cpu.read_f64(instruction.rs2.value());
            let rs3_val = cpu.read_f64(instruction.func7.value() >> 2);

            let res = (rs1_val * rs2_val) - rs3_val;

            cpu.write_f64(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC2_MASK,
        bits: 0b1001011 | 0b01 << FUNC2_POS,
        name: "FNMSUB.S",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f64(instruction.rs1.value());
            let rs2_val = cpu.read_f64(instruction.rs2.value());
            let rs3_val = cpu.read_f64(instruction.func7.value() >> 2);

            let res = -(rs1_val * rs2_val) - rs3_val;

            cpu.write_f64(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC2_MASK,
        bits: 0b1001111 | 0b01 << FUNC2_POS,
        name: "FNMADD.S",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f64(instruction.rs1.value());
            let rs2_val = cpu.read_f64(instruction.rs2.value());
            let rs3_val = cpu.read_f64(instruction.func7.value() >> 2);

            let res = -(rs1_val * rs2_val) + rs3_val;

            cpu.write_f64(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b010 << FUNC3_POS | 0b1010000 << FUNC7_POS,
        name: "FEQ.S",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f32(instruction.rs1.value());
            let rs2_val = cpu.read_f32(instruction.rs2.value());
            let res = (rs1_val == rs2_val) as u32;

            cpu.write_x_u32(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b010 << FUNC3_POS | 0b1010001 << FUNC7_POS,
        name: "FEQ.D",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f64(instruction.rs1.value());
            let rs2_val = cpu.read_f64(instruction.rs2.value());
            let res = (rs1_val == rs2_val) as u64;

            cpu.write_x_u64(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b001 << FUNC3_POS | 0b1010000 << FUNC7_POS,
        name: "FLT.S",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f32(instruction.rs1.value());
            let rs2_val = cpu.read_f32(instruction.rs2.value());
            let res = (rs1_val < rs2_val) as u32;

            cpu.write_x_u32(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b001 << FUNC3_POS | 0b1010001 << FUNC7_POS,
        name: "FLT.D",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f64(instruction.rs1.value());
            let rs2_val = cpu.read_f64(instruction.rs2.value());
            let res = (rs1_val < rs2_val) as u64;

            cpu.write_x_u64(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b000 << FUNC3_POS | 0b1010000 << FUNC7_POS,
        name: "FLE.S",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f32(instruction.rs1.value());
            let rs2_val = cpu.read_f32(instruction.rs2.value());
            let res = (rs1_val <= rs2_val) as u32;

            cpu.write_x_u32(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b000 << FUNC3_POS | 0b1010001 << FUNC7_POS,
        name: "FLE.D",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f64(instruction.rs1.value());
            let rs2_val = cpu.read_f64(instruction.rs2.value());
            let res = (rs1_val <= rs2_val) as u64;

            cpu.write_x_u64(instruction.rd.value(), res);

            Ok(())
        },
    },
];
