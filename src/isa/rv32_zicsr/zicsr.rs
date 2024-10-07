use crate::types::*;

use anyhow::Ok;

pub const RV32_ZICSR_SET: [Instruction; 6] = [
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b001 << FUNC3_POS | 0b1110011,
        name: "CSRRW",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let rs1_value = cpu.read_x_u32(instruction.rs1.value())?;
            let csr_addr = instruction.imm;
            let old_csr_value = cpu.csr_table.read32(csr_addr);

            cpu.write_x_u32(instruction.rd.value(), old_csr_value)?;
            cpu.csr_table.write32(csr_addr, rs1_value);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b010 << FUNC3_POS | 0b1110011,
        name: "CSRRS",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let rs1_value = cpu.read_x_u32(instruction.rs1.value())?;
            let old_csr_value = cpu.csr_table.read32(instruction.imm);

            cpu.write_x_u32(instruction.rd.value(), old_csr_value)?;
            cpu.csr_table
                .write32(instruction.imm, old_csr_value | rs1_value);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b011 << FUNC3_POS | 0b1110011,
        name: "CSRRC",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let rs1_value = cpu.read_x_u32(instruction.rs1.value())?;
            let old_csr_value = cpu.csr_table.read32(instruction.imm);

            cpu.write_x_u32(instruction.rd.value(), old_csr_value)?;
            cpu.csr_table
                .write32(instruction.imm, old_csr_value & !rs1_value);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b101 << FUNC3_POS | 0b1110011,
        name: "CSRRWI",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let imm_value = instruction.rs1.value() as u32;
            let old_csr_value = cpu.csr_table.read32(instruction.imm);

            cpu.write_x_u32(instruction.rd.value(), old_csr_value)?;
            cpu.csr_table.write32(instruction.imm, imm_value);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b110 << FUNC3_POS | 0b1110011,
        name: "CSRRSI",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let imm_value = instruction.rs1.value() as u32;
            let old_csr_value = cpu.csr_table.read32(instruction.imm);

            cpu.write_x_u32(instruction.rd.value(), old_csr_value)?;
            cpu.csr_table
                .write32(instruction.imm, old_csr_value | imm_value);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b111 << FUNC3_POS | 0b1110011,
        name: "CSRRCI",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let imm_value = instruction.rs1.value() as u32;
            let old_csr_value = cpu.csr_table.read32(instruction.imm);

            cpu.write_x_u32(instruction.rd.value(), old_csr_value)?;
            cpu.csr_table
                .write32(instruction.imm, old_csr_value & !imm_value);

            Ok(())
        },
    },
];
