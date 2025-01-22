use crate::{
    types::*,
    utils::binary_utils::{f32_to_u32, f64_to_u64, u32_to_f32, u64_to_f64},
};

use anyhow::Ok;

pub const RV64F_SET_MOV: [Instruction; 10] = [
    Instruction {
        mask: OPCODE_MASK | RS2_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b1110001 << FUNC7_POS,
        name: "FMV.X.D",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f64(instruction.rs1.value());
            let res = f64_to_u64(rs1_val);

            cpu.write_x_u64(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | RS2_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b1111000 << FUNC7_POS,
        name: "FMV.X.W",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f32(instruction.rs1.value());
            let res = f32_to_u32(rs1_val);

            cpu.write_x_u32(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | RS2_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b1111001 << FUNC7_POS,
        name: "FMV.D.X",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_x_u64(instruction.rs1.value());
            let res = u64_to_f64(rs1_val);

            cpu.write_f64(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | RS2_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b1110000 << FUNC7_POS,
        name: "FMV.W.X",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_x_u32(instruction.rs1.value());
            let res = u32_to_f32(rs1_val);

            cpu.write_f32(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b000 << FUNC3_POS | 0b0010000 << FUNC7_POS,
        name: "FSGNJ.S",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f32(instruction.rs1.value());
            let rs2_val = cpu.read_f32(instruction.rs2.value());

            let rs1_bits = f32_to_u32(rs1_val);
            let rs2_bits = f32_to_u32(rs2_val);

            let res = u32_to_f32(
                (rs1_bits & 0x7FFF_FFFF)  // Clear rs1's sign bit
                    | (rs2_bits & 0x8000_0000), // Keep rs2's sign bit
            );

            cpu.write_f32(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b001 << FUNC3_POS | 0b0010000 << FUNC7_POS,
        name: "FSGNJN.S",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f32(instruction.rs1.value());
            let rs2_val = cpu.read_f32(instruction.rs2.value());

            let rs1_bits = f32_to_u32(rs1_val);
            let rs2_bits = f32_to_u32(rs2_val);

            let res = u32_to_f32(
                (rs1_bits & 0x7FFF_FFFF)  // Clear rs1's sign bit
                    | (!rs2_bits & 0x8000_0000), // Keep rs2's inverted sign bit
            );

            cpu.write_f32(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b010 << FUNC3_POS | 0b0010000 << FUNC7_POS,
        name: "FSGNJX.S",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f32(instruction.rs1.value());
            let rs2_val = cpu.read_f32(instruction.rs2.value());

            let rs1_bits = f32_to_u32(rs1_val);
            let rs2_bits = f32_to_u32(rs2_val);

            let res = u32_to_f32(
                (rs1_bits & 0x7FFF_FFFF)  // Clear rs1's sign bit
                    | ((rs1_bits ^ rs2_bits) & 0x8000_0000), // rs1 XOR rs2 and keep sign bit
            );

            cpu.write_f32(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b000 << FUNC3_POS | 0b0010001 << FUNC7_POS,
        name: "FSGNJ.D",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f64(instruction.rs1.value());
            let rs2_val = cpu.read_f64(instruction.rs2.value());

            let rs1_bits = f64_to_u64(rs1_val);
            let rs2_bits = f64_to_u64(rs2_val);

            let res = u64_to_f64(
                (rs1_bits & 0x7FFF_FFFF_FFFF_FFFF)  // Clear rs1's sign bit
                    | (rs2_bits & 0x8000_0000_0000_0000), // Keep rs2's sign bit
            );

            cpu.write_f64(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b001 << FUNC3_POS | 0b0010001 << FUNC7_POS,
        name: "FSGNJN.D",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f64(instruction.rs1.value());
            let rs2_val = cpu.read_f64(instruction.rs2.value());

            let rs1_bits = f64_to_u64(rs1_val);
            let rs2_bits = f64_to_u64(rs2_val);

            let res = u64_to_f64(
                (rs1_bits & 0x7FFF_FFFF_FFFF_FFFF)  // Clear rs1's sign bit
                    | (!rs2_bits & 0x8000_0000_0000_0000), // Keep rs2's inverted sign bit
            );

            cpu.write_f64(instruction.rd.value(), res);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b1010011 | 0b010 << FUNC3_POS | 0b0010001 << FUNC7_POS,
        name: "FSGNJX.D",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let rs1_val = cpu.read_f64(instruction.rs1.value());
            let rs2_val = cpu.read_f64(instruction.rs2.value());

            let rs1_bits = f64_to_u64(rs1_val);
            let rs2_bits = f64_to_u64(rs2_val);

            let res = u64_to_f64(
                (rs1_bits & 0x7FFF_FFFF_FFFF_FFFF)  // Clear rs1's sign bit
                    | ((rs1_bits ^ rs2_bits) & 0x8000_0000_0000_0000), // rs1 XOR rs2 and keep sign bit
            );

            cpu.write_f64(instruction.rd.value(), res);

            Ok(())
        },
    },
];
