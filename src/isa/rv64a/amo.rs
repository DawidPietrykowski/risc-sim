use crate::{
    types::{
        parse_instruction_r, BitValue, Instruction, InstructionType, FUNC3_MASK, FUNC3_POS,
        FUNC7_MASK, FUNC7_POS, OPCODE_MASK,
    },
    utils::binary_utils::{
        i32_to_u32, i64_to_u64, sign_extend_32bit_to_64bit, u32_to_i32, u64_to_i64,
    },
};

pub const RV64A_SET_AMO: [Instruction; 10] = [
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | (FUNC7_MASK & !(0b11 << (FUNC7_POS))),
        bits: 0b0101111 | 0b011 << FUNC3_POS | 0b00001 << (FUNC7_POS + 2),
        name: "AMOSWAP.D",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let addr = cpu.read_x_u64(instruction.rs1.value());

            let data = cpu.read_mem_u64(addr)?;

            let rs2 = cpu.read_x_u64(instruction.rs2.value());

            cpu.write_x_u64(instruction.rd.value(), data);

            let res = rs2;
            cpu.write_mem_u64(addr, res)?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | (FUNC7_MASK & !(0b11 << (FUNC7_POS))),
        bits: 0b0101111 | 0b010 << FUNC3_POS | 0b00001 << (FUNC7_POS + 2),
        name: "AMOSWAP.W",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let addr = cpu.read_x_u64(instruction.rs1.value());

            let data = cpu.read_mem_u32(addr)?;

            let rs2 = cpu.read_x_u64(instruction.rs2.value()) as u32;

            cpu.write_x_i64(instruction.rd.value(), sign_extend_32bit_to_64bit(data));

            let res = rs2;
            cpu.write_mem_u32(addr, res)?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | (FUNC7_MASK & !(0b11 << (FUNC7_POS))),
        bits: 0b0101111 | 0b011 << FUNC3_POS | 0b00000 << (FUNC7_POS + 2),
        name: "AMOADD.D",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let addr = cpu.read_x_u64(instruction.rs1.value());

            let data = u64_to_i64(cpu.read_mem_u64(addr)?);

            let rs2 = cpu.read_x_i64(instruction.rs2.value());

            cpu.write_x_i64(instruction.rd.value(), data);

            let (res, _) = data.overflowing_add(rs2);
            cpu.write_mem_u64(addr, i64_to_u64(res))?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | (FUNC7_MASK & !(0b11 << (FUNC7_POS))),
        bits: 0b0101111 | 0b010 << FUNC3_POS | 0b00000 << (FUNC7_POS + 2),
        name: "AMOADD.W",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let addr = cpu.read_x_u64(instruction.rs1.value());

            let data = u32_to_i32(cpu.read_mem_u32(addr)?);

            let rs2 = cpu.read_x_i64(instruction.rs2.value()) as i32;

            cpu.write_x_i64(instruction.rd.value(), data as i64);

            let (res, _) = data.overflowing_add(rs2);
            cpu.write_mem_u32(addr, i32_to_u32(res))?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | (FUNC7_MASK & !(0b11 << (FUNC7_POS))),
        bits: 0b0101111 | 0b011 << FUNC3_POS | 0b00100 << (FUNC7_POS + 2),
        name: "AMOXOR.D",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let addr = cpu.read_x_u64(instruction.rs1.value());

            let data = cpu.read_mem_u64(addr)?;

            let rs2 = cpu.read_x_u64(instruction.rs2.value());

            cpu.write_x_u64(instruction.rd.value(), data);

            let res = data ^ rs2;
            cpu.write_mem_u64(addr, res)?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | (FUNC7_MASK & !(0b11 << (FUNC7_POS))),
        bits: 0b0101111 | 0b010 << FUNC3_POS | 0b00100 << (FUNC7_POS + 2),
        name: "AMOXOR.W",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let addr = cpu.read_x_u64(instruction.rs1.value());

            let data = cpu.read_mem_u32(addr)?;

            let rs2 = cpu.read_x_u64(instruction.rs2.value()) as u32;

            cpu.write_x_i64(instruction.rd.value(), sign_extend_32bit_to_64bit(data));

            let res = data ^ rs2;
            cpu.write_mem_u32(addr, res)?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | (FUNC7_MASK & !(0b11 << (FUNC7_POS))),
        bits: 0b0101111 | 0b011 << FUNC3_POS | 0b01100 << (FUNC7_POS + 2),
        name: "AMOAND.D",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let addr = cpu.read_x_u64(instruction.rs1.value());

            let data = cpu.read_mem_u64(addr)?;

            let rs2 = cpu.read_x_u64(instruction.rs2.value());

            cpu.write_x_u64(instruction.rd.value(), data);

            let res = data & rs2;
            cpu.write_mem_u64(addr, res)?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | (FUNC7_MASK & !(0b11 << (FUNC7_POS))),
        bits: 0b0101111 | 0b010 << FUNC3_POS | 0b01100 << (FUNC7_POS + 2),
        name: "AMOAND.W",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let addr = cpu.read_x_u64(instruction.rs1.value());

            let data = cpu.read_mem_u32(addr)?;

            let rs2 = cpu.read_x_u64(instruction.rs2.value()) as u32;

            cpu.write_x_i64(instruction.rd.value(), sign_extend_32bit_to_64bit(data));

            let res = data & rs2;
            cpu.write_mem_u32(addr, res)?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | (FUNC7_MASK & !(0b11 << (FUNC7_POS))),
        bits: 0b0101111 | 0b011 << FUNC3_POS | 0b01000 << (FUNC7_POS + 2),
        name: "AMOOR.D",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let addr = cpu.read_x_u64(instruction.rs1.value());

            let data = cpu.read_mem_u64(addr)?;

            let rs2 = cpu.read_x_u64(instruction.rs2.value());

            cpu.write_x_u64(instruction.rd.value(), data);

            let res = data | rs2;
            cpu.write_mem_u64(addr, res)?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | (FUNC7_MASK & !(0b11 << (FUNC7_POS))),
        bits: 0b0101111 | 0b010 << FUNC3_POS | 0b01000 << (FUNC7_POS + 2),
        name: "AMOOR.W",
        instruction_type: InstructionType::R,
        operation: |cpu, word| {
            let instruction = parse_instruction_r(word);

            let addr = cpu.read_x_u64(instruction.rs1.value());

            let data = cpu.read_mem_u32(addr)?;

            let rs2 = cpu.read_x_u64(instruction.rs2.value()) as u32;

            cpu.write_x_i64(instruction.rd.value(), sign_extend_32bit_to_64bit(data));

            let res = data | rs2;
            cpu.write_mem_u32(addr, res)?;

            Ok(())
        },
    },
];
