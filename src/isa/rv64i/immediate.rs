use crate::types::*;
use crate::utils::binary_utils::*;

use anyhow::Ok;

pub const RV64I_SET_I: [Instruction; 15] = [
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b000 << FUNC3_POS | 0b0010011,
        name: "ADDI",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let imm = sign_extend_12bit_to_64bit(instruction.imm.value());
            let rs1 = cpu.read_x_i64(instruction.rs1.value())?;
            let (res, _) = imm.overflowing_add(rs1);
            cpu.write_x_i64(instruction.rd.value(), res)?;

            cpu.debug_print(|| {
                format!(
                    "ADDI: r{}({:#x}) = r{}({}) + {}",
                    instruction.rd.value(),
                    res,
                    instruction.rs1.value(),
                    rs1,
                    imm
                )
            });
            cpu.debug_print(|| format!("rd: {}", cpu.read_x_i64(instruction.rd.value()).unwrap()));
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b000 << FUNC3_POS | 0b0011011,
        name: "ADDIW",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let imm = sign_extend_12bit_to_32bit(instruction.imm.value());
            let rs1 = cpu.read_x_i64(instruction.rs1.value())? as i32;
            let (res, _) = imm.overflowing_add(rs1);
            cpu.write_x_i64(instruction.rd.value(), res as i64)?;

            cpu.debug_print(|| {
                format!(
                    "ADDI: r{}({:#x}) = r{}({}) + {}",
                    instruction.rd.value(),
                    res,
                    instruction.rs1.value(),
                    rs1,
                    imm
                )
            });
            cpu.debug_print(|| format!("rd: {}", cpu.read_x_i64(instruction.rd.value()).unwrap()));
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b010 << FUNC3_POS | 0b0010011,
        name: "SLTI",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let imm = sign_extend_12bit_to_64bit(instruction.imm.value());
            let rs1 = cpu.read_x_i64(instruction.rs1.value())?;
            cpu.write_x_i64(instruction.rd.value(), if rs1 < imm { 1 } else { 0 })?;
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b011 << FUNC3_POS | 0b0010011,
        name: "SLTIU",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let imm = i64_to_u64(sign_extend_12bit_to_64bit(instruction.imm.value()));
            let rs1 = cpu.read_x_u64(instruction.rs1.value())?;
            cpu.write_x_i64(instruction.rd.value(), if rs1 < imm { 1 } else { 0 })?;
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b111 << FUNC3_POS | 0b0010011,
        name: "ANDI",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let imm = sign_extend_12bit_to_64bit(instruction.imm.value());
            let rs1 = cpu.read_x_i64(instruction.rs1.value())?;
            cpu.debug_print(|| {
                format!(
                    "ANDI: r{}({:#x}) = r{}({}) & {}",
                    instruction.rd.value(),
                    rs1 & imm,
                    instruction.rs1.value(),
                    rs1,
                    imm
                )
            });
            cpu.write_x_i64(instruction.rd.value(), rs1 & imm)?;
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: (FUNC3_ORI as u32) << FUNC3_POS | 0b0010011,
        name: "ORI",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let imm = sign_extend_12bit_to_64bit(instruction.imm.value());
            let rs1 = cpu.read_x_i64(instruction.rs1.value())?;
            cpu.write_x_i64(instruction.rd.value(), rs1 | imm)?;
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: (FUNC3_XORI as u32) << FUNC3_POS | 0b0010011,
        name: "XORI",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let imm = sign_extend_12bit_to_64bit(instruction.imm.value());
            let rs1 = cpu.read_x_i64(instruction.rs1.value())?;
            cpu.write_x_i64(instruction.rd.value(), rs1 ^ imm)?;
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | TOP6_MASK,
        bits: 0b000000 << TOP6_POS | 0b001 << FUNC3_POS | 0b0010011,
        name: "SLLI",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let shamt = (instruction.imm.value() & (U6_MASK as u16)) as u64;
            let res: u64 = cpu.read_x_u64(instruction.rs1.value())? << shamt;
            cpu.write_x_u64(instruction.rd.value(), res)?;
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0000000 << FUNC7_POS | 0b001 << FUNC3_POS | 0b0011011,
        name: "SLLIW",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let shamt = (instruction.imm.value() & (U5_MASK as u16)) as u32;
            let res: u32 = (cpu.read_x_u64(instruction.rs1.value())? as u32) << shamt;
            cpu.write_x_i64(instruction.rd.value(), sign_extend_32bit_to_64bit(res))?;
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | TOP6_MASK,
        bits: 0b000000 << TOP6_POS | 0b101 << FUNC3_POS | 0b0010011,
        name: "SRLI",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let shamt = (instruction.imm.value() & (U6_MASK as u16)) as u64;
            let res: u64 = cpu.read_x_u64(instruction.rs1.value())? >> shamt;
            cpu.write_x_u64(instruction.rd.value(), res)?;
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0000000 << FUNC7_POS | 0b101 << FUNC3_POS | 0b0011011,
        name: "SRLIW",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let shamt = (instruction.imm.value() & (U5_MASK as u16)) as u32;
            let res: u32 = (cpu.read_x_u64(instruction.rs1.value())? as u32) >> shamt;
            cpu.write_x_i64(instruction.rd.value(), sign_extend_32bit_to_64bit(res))?;
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | TOP6_MASK,
        bits: 0b010000 << TOP6_POS | 0b101 << FUNC3_POS | 0b0010011,
        name: "SRAI",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let shamt = (instruction.imm.value() & (U6_MASK as u16)) as u64;
            let res: i64 = cpu.read_x_i64(instruction.rs1.value())? >> shamt;
            cpu.write_x_i64(instruction.rd.value(), res)?;
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b010000 << FUNC7_POS | 0b101 << FUNC3_POS | 0b0011011,
        name: "SRAIW",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let shamt = (instruction.imm.value() & (U5_MASK as u16)) as u32;
            let res: i32 = (cpu.read_x_i64(instruction.rs1.value())? as i32) >> shamt;
            cpu.write_x_i64(instruction.rd.value(), res as i64)?;
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK,
        bits: 0b0110111,
        name: "LUI",
        instruction_type: InstructionType::U,
        operation: |cpu, word| {
            let instruction = parse_instruction_u(word);
            cpu.write_x_i64(
                instruction.rd.value(),
                sign_extend_32bit_to_64bit(instruction.imm),
            )?;
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK,
        bits: 0b0010111,
        name: "AUIPC",
        instruction_type: InstructionType::U,
        operation: |cpu, word| {
            let instruction = parse_instruction_u(word);
            let res: u64 = (sign_extend_32bit_to_64bit(instruction.imm) as u64)
                .wrapping_add(cpu.read_current_instruction_addr_u64());

            cpu.write_x_u64(instruction.rd.value(), res)?;
            Ok(())
        },
    },
];
