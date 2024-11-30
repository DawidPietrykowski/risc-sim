use crate::{isa::csr::csr_types::CSRAddress, types::*};

use anyhow::Ok;

pub const RV64_ZICSR_SET: [Instruction; 6] = [
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b001 << FUNC3_POS | 0b1110011,
        name: "CSRRW",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let rs1_value = cpu.read_x_u64(instruction.rs1.value())?;
            let csr_addr = instruction.imm;
            let old_csr_value = cpu.csr_table.read64(csr_addr);

            cpu.write_x_u64(instruction.rd.value(), old_csr_value)?;
            cpu.csr_table.write64(csr_addr, rs1_value);

            // TODO: Remove
            if csr_addr == CSRAddress::Satp.as_u12() {
                println!(
                    "Satp: {:#018x} (PPN={:#010x}, ASID={:#06x}, MODE={:#04x}) PC: {:#x}",
                    rs1_value,
                    rs1_value >> 44,
                    (rs1_value >> 44) & 0xfff,
                    rs1_value & 0xf,
                    cpu.current_instruction_pc_64
                );
                if rs1_value != 0 {
                    // test kernel address 
                    const KERNEL_ADDR: u64 = 0x80000000;
                    let res = cpu.translate_address_if_needed(KERNEL_ADDR)?;
                    println!("Kernel address {:#x} translated to {:#x}", KERNEL_ADDR, res);
                }
            }
            
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
            let rs1_value = cpu.read_x_u64(instruction.rs1.value())?;
            let old_csr_value = cpu.csr_table.read64(instruction.imm);

            cpu.write_x_u64(instruction.rd.value(), old_csr_value)?;
            cpu.csr_table
                .write64(instruction.imm, old_csr_value | rs1_value);

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
            let rs1_value = cpu.read_x_u64(instruction.rs1.value())?;
            let old_csr_value = cpu.csr_table.read64(instruction.imm);

            cpu.write_x_u64(instruction.rd.value(), old_csr_value)?;
            cpu.csr_table
                .write64(instruction.imm, old_csr_value & !rs1_value);

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
            let imm_value = instruction.rs1.value() as u64;
            let old_csr_value = cpu.csr_table.read64(instruction.imm);

            cpu.write_x_u64(instruction.rd.value(), old_csr_value)?;
            cpu.csr_table.write64(instruction.imm, imm_value);

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
            let imm_value = instruction.rs1.value() as u64;
            let old_csr_value = cpu.csr_table.read64(instruction.imm);

            cpu.write_x_u64(instruction.rd.value(), old_csr_value)?;
            cpu.csr_table
                .write64(instruction.imm, old_csr_value | imm_value);

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
            let imm_value = instruction.rs1.value() as u64;
            let old_csr_value = cpu.csr_table.read64(instruction.imm);

            cpu.write_x_u64(instruction.rd.value(), old_csr_value)?;
            cpu.csr_table
                .write64(instruction.imm, old_csr_value & !imm_value);

            Ok(())
        },
    },
];
