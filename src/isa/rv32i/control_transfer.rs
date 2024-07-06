use crate::isa::types::*;

use anyhow::Ok;

pub const RV32I_SET_UJ: [Instruction; 8] = [
    Instruction {
        mask: OPCODE_MASK,
        bits: 0b1101111,
        name: "JAL",
        instruction_type: InstructionType::UJ,
        operation: |cpu, word| {
            let instruction = parse_instruction_uj(word);

            cpu.write_x_u32(instruction.rd.value(), cpu.read_pc_u32())?;

            let extended_offset = instruction.imm.as_i32();
            let moved_pc = cpu
                .read_current_instruction_addr_u32()
                .wrapping_add_signed(extended_offset);
            cpu.write_pc_u32(moved_pc);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b1100111,
        name: "JALR",
        instruction_type: InstructionType::I,
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);

            let offset = instruction.imm.as_i32();
            let result = cpu
                .read_x_u32(instruction.rs1.value())?
                .wrapping_add_signed(offset)
                & !(0b1);

            cpu.write_x_u32(instruction.rd.value(), result)?;
            cpu.write_pc_u32(result);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b1100011 | 0b000 << FUNC3_POS,
        name: "BEQ",
        instruction_type: InstructionType::SB,
        operation: |cpu, word| {
            let instruction = parse_instruction_sb(word);

            let rs1 = cpu.read_x_i32(instruction.rs1.value())?;
            let rs2 = cpu.read_x_i32(instruction.rs2.value())?;

            if rs1 == rs2 {
                let extended_offset = instruction.imm.as_i32();
                let moved_pc = cpu
                    .read_current_instruction_addr_u32()
                    .wrapping_add_signed(extended_offset);
                cpu.write_pc_u32(moved_pc);
            }

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b1100011 | 0b001 << FUNC3_POS,
        name: "BNE",
        instruction_type: InstructionType::SB,
        operation: |cpu, word| {
            let instruction = parse_instruction_sb(word);

            let rs1 = cpu.read_x_i32(instruction.rs1.value())?;
            let rs2 = cpu.read_x_i32(instruction.rs2.value())?;

            if rs1 != rs2 {
                let extended_offset = instruction.imm.as_i32();
                let moved_pc = cpu
                    .read_current_instruction_addr_u32()
                    .wrapping_add_signed(extended_offset);
                cpu.write_pc_u32(moved_pc);
            }

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b1100011 | 0b100 << FUNC3_POS,
        name: "BLT",
        instruction_type: InstructionType::SB,
        operation: |cpu, word| {
            let instruction = parse_instruction_sb(word);

            let rs1 = cpu.read_x_i32(instruction.rs1.value())?;
            let rs2 = cpu.read_x_i32(instruction.rs2.value())?;

            if rs1 < rs2 {
                let extended_offset = instruction.imm.as_i32();
                let moved_pc = cpu
                    .read_current_instruction_addr_u32()
                    .wrapping_add_signed(extended_offset);
                cpu.write_pc_u32(moved_pc);
            }

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b1100011 | 0b101 << FUNC3_POS,
        name: "BGE",
        instruction_type: InstructionType::SB,
        operation: |cpu, word| {
            let instruction = parse_instruction_sb(word);

            let rs1 = cpu.read_x_i32(instruction.rs1.value())?;
            let rs2 = cpu.read_x_i32(instruction.rs2.value())?;

            if rs1 > rs2 {
                let extended_offset = instruction.imm.as_i32();
                let moved_pc = cpu
                    .read_current_instruction_addr_u32()
                    .wrapping_add_signed(extended_offset);
                cpu.write_pc_u32(moved_pc);
            }

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b1100011 | 0b110 << FUNC3_POS,
        name: "BLTU",
        instruction_type: InstructionType::SB,
        operation: |cpu, word| {
            let instruction = parse_instruction_sb(word);

            let rs1 = cpu.read_x_u32(instruction.rs1.value())?;
            let rs2 = cpu.read_x_u32(instruction.rs2.value())?;

            if rs1 <= rs2 {
                let extended_offset = instruction.imm.as_i32();
                let moved_pc = cpu
                    .read_current_instruction_addr_u32()
                    .wrapping_add_signed(extended_offset);
                cpu.write_pc_u32(moved_pc);
            }

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b1100011 | 0b111 << FUNC3_POS,
        name: "BGEU",
        instruction_type: InstructionType::SB,
        operation: |cpu, word| {
            let instruction = parse_instruction_sb(word);

            let rs1 = cpu.read_x_u32(instruction.rs1.value())?;
            let rs2 = cpu.read_x_u32(instruction.rs2.value())?;

            if rs1 >= rs2 {
                let extended_offset = instruction.imm.as_i32();
                let moved_pc = cpu
                    .read_current_instruction_addr_u32()
                    .wrapping_add_signed(extended_offset);
                cpu.write_pc_u32(moved_pc);
            }

            Ok(())
        },
    },
];
