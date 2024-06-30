use crate::isa::types::*;

use anyhow::Ok;

pub const RV32I_SET_UJ: [Instruction; 8] = [
    Instruction {
        mask: OPCODE_MASK,
        bits: 0b1101111,
        name: "JAL",
        operation: |cpu, word| {
            let instruction = parse_instruction_uj(word);

            let extended_offset = instruction.imm.as_i32();
            let moved_pc = cpu.read_pc_u32().wrapping_add_signed(extended_offset);
            cpu.write_pc_u32(moved_pc);

            cpu.write_x_u32(instruction.rd.value(), moved_pc + 4)?;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b1100111,
        name: "JALR",
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
        operation: |cpu, word| {
            let instruction = parse_instruction_sb(word);

            let rs1 = cpu.read_x_i32(instruction.rs1.value())?;
            let rs2 = cpu.read_x_i32(instruction.rs2.value())?;

            if rs1 == rs2 {
                cpu.write_pc_u32(instruction.imm.as_u32());
            }

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b1100011 | 0b001 << FUNC3_POS,
        name: "BNE",
        operation: |cpu, word| {
            let instruction = parse_instruction_sb(word);

            let rs1 = cpu.read_x_i32(instruction.rs1.value())?;
            let rs2 = cpu.read_x_i32(instruction.rs2.value())?;

            if rs1 != rs2 {
                cpu.write_pc_u32(instruction.imm.as_u32());
            }

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b1100011 | 0b100 << FUNC3_POS,
        name: "BLT",
        operation: |cpu, word| {
            let instruction = parse_instruction_sb(word);

            let rs1 = cpu.read_x_i32(instruction.rs1.value())?;
            let rs2 = cpu.read_x_i32(instruction.rs2.value())?;

            if rs1 < rs2 {
                cpu.write_pc_u32(instruction.imm.as_u32());
            }

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b1100011 | 0b101 << FUNC3_POS,
        name: "BGE",
        operation: |cpu, word| {
            let instruction = parse_instruction_sb(word);

            let rs1 = cpu.read_x_i32(instruction.rs1.value())?;
            let rs2 = cpu.read_x_i32(instruction.rs2.value())?;

            if rs1 > rs2 {
                cpu.write_pc_u32(instruction.imm.as_u32());
            }

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b1100011 | 0b110 << FUNC3_POS,
        name: "BLTU",
        operation: |cpu, word| {
            let instruction = parse_instruction_sb(word);

            let rs1 = cpu.read_x_u32(instruction.rs1.value())?;
            let rs2 = cpu.read_x_u32(instruction.rs2.value())?;

            if rs1 <= rs2 {
                cpu.write_pc_u32(instruction.imm.as_u32());
            }

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b1100011 | 0b111 << FUNC3_POS,
        name: "BGEU",
        operation: |cpu, word| {
            let instruction = parse_instruction_sb(word);

            let rs1 = cpu.read_x_u32(instruction.rs1.value())?;
            let rs2 = cpu.read_x_u32(instruction.rs2.value())?;

            if rs1 >= rs2 {
                cpu.write_pc_u32(instruction.imm.as_u32());
            }

            Ok(())
        },
    },
];
