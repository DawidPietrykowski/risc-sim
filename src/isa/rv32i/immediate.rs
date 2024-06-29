use crate::isa::cpu::{Cpu, Operation};
use crate::isa::types::*;
use crate::utils::binary_utils::*;

use anyhow::{Ok, Result};

pub struct AddI {
    instruction: IInstructionData,
}

impl Operation<IInstructionData> for AddI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let imm = sign_extend_12bit_to_32bit(self.instruction.imm);
        let rs1 = cpu.read_x_i32(self.instruction.rs1.value())?;
        let (res, _) = imm.overflowing_add(rs1);
        cpu.write_x_i32(self.instruction.rd.value(), res)?;
        Ok(())
    }

    fn new(instruction: IInstructionData) -> Self {
        AddI {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &IInstructionData {
        &self.instruction
    }
}
pub struct SLTI {
    instruction: IInstructionData,
}

impl Operation<IInstructionData> for SLTI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let imm = sign_extend_12bit_to_32bit(self.instruction.imm);
        let rs1 = cpu.read_x_i32(self.instruction.rs1.value())?;
        cpu.write_x_i32(self.instruction.rd.value(), if rs1 < imm { 1 } else { 0 })?;
        Ok(())
    }

    fn new(instruction: IInstructionData) -> Self {
        SLTI {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &IInstructionData {
        &self.instruction
    }
}

pub struct ANDI {
    instruction: IInstructionData,
}

impl Operation<IInstructionData> for ANDI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let imm = sign_extend_12bit_to_32bit(self.instruction.imm);
        let rs1 = cpu.read_x_i32(self.instruction.rs1.value())?;
        cpu.write_x_i32(self.instruction.rd.value(), rs1 & imm)?;
        Ok(())
    }

    fn new(instruction: IInstructionData) -> Self {
        ANDI {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &IInstructionData {
        &self.instruction
    }
}

pub struct ORI {
    instruction: IInstructionData,
}

impl Operation<IInstructionData> for ORI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let imm = sign_extend_12bit_to_32bit(self.instruction.imm);
        let rs1 = cpu.read_x_i32(self.instruction.rs1.value())?;
        cpu.write_x_i32(self.instruction.rd.value(), rs1 | imm)?;
        Ok(())
    }

    fn new(instruction: IInstructionData) -> Self {
        ORI {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &IInstructionData {
        &self.instruction
    }
}

pub struct XORI {
    instruction: IInstructionData,
}

impl Operation<IInstructionData> for XORI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let imm = sign_extend_12bit_to_32bit(self.instruction.imm);
        let rs1 = cpu.read_x_i32(self.instruction.rs1.value())?;
        cpu.write_x_i32(self.instruction.rd.value(), rs1 ^ imm)?;
        Ok(())
    }

    fn new(instruction: IInstructionData) -> Self {
        XORI {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &IInstructionData {
        &self.instruction
    }
}

pub struct SLLI {
    instruction: IInstructionData,
}

impl Operation<IInstructionData> for SLLI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let shamt = (self.instruction.imm & 0b11111) as u32;
        let res: u32 = cpu.read_x_u32(self.instruction.rs1.value())? << shamt;
        cpu.write_x_u32(self.instruction.rd.value(), res)?;
        Ok(())
    }

    fn new(instruction: IInstructionData) -> Self {
        SLLI {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &IInstructionData {
        &self.instruction
    }
}

pub struct SRLI {
    instruction: IInstructionData,
}

impl Operation<IInstructionData> for SRLI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let shamt = (self.instruction.imm & 0b11111) as u32;
        let res: u32 = cpu.read_x_u32(self.instruction.rs1.value())? >> shamt;
        cpu.write_x_u32(self.instruction.rd.value(), res)?;
        Ok(())
    }

    fn new(instruction: IInstructionData) -> Self {
        SRLI {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &IInstructionData {
        &self.instruction
    }
}

pub struct SRAI {
    instruction: IInstructionData,
}

impl Operation<IInstructionData> for SRAI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let shamt = (self.instruction.imm & 0b11111) as u32;
        let res: i32 = cpu.read_x_i32(self.instruction.rs1.value())? >> shamt;
        cpu.write_x_i32(self.instruction.rd.value(), res)?;
        Ok(())
    }

    fn new(instruction: IInstructionData) -> Self {
        SRAI {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &IInstructionData {
        &self.instruction
    }
}

pub struct LUI {
    instruction: UInstructionData,
}

impl Operation<UInstructionData> for LUI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let shifted_imm = self.instruction.imm << 12;
        cpu.write_x_u32(self.instruction.rd.value(), shifted_imm)?;
        Ok(())
    }

    fn new(instruction: UInstructionData) -> Self {
        LUI {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &UInstructionData {
        &self.instruction
    }
}

pub struct AUIPC {
    instruction: UInstructionData,
}

impl Operation<UInstructionData> for AUIPC {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let res: u32 = (self.instruction.imm << 12).wrapping_add(cpu.read_pc_u32());

        cpu.write_pc_u32(res);
        cpu.set_skip_pc_increment_flag(); // Disable default pc increment logic

        cpu.write_x_u32(self.instruction.rd.value(), res)?;
        Ok(())
    }

    fn new(instruction: UInstructionData) -> Self {
        AUIPC {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &UInstructionData {
        &self.instruction
    }
}

pub const RV32I_SET: [Instruction; 10] = [
    Instruction{
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b000 << FUNC3_POS | 0b0010011,
        name: "ADDI",
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let imm = sign_extend_12bit_to_32bit(instruction.imm);
            let rs1 = cpu.read_x_i32(instruction.rs1.value())?;
            let (res, _) = imm.overflowing_add(rs1);
            cpu.write_x_i32(instruction.rd.value(), res)?;
            Ok(())
        }
    },
    Instruction{
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b010 << FUNC3_POS | 0b0010011,
        name: "SLTI",
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let imm = sign_extend_12bit_to_32bit(instruction.imm);
            let rs1 = cpu.read_x_i32(instruction.rs1.value())?;
            cpu.write_x_i32(instruction.rd.value(), if rs1 < imm { 1 } else { 0 })?;
            Ok(())
        }
    },
    Instruction{
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: 0b111 << FUNC3_POS | 0b0010011,
        name: "ANDI",
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let imm = sign_extend_12bit_to_32bit(instruction.imm);
            let rs1 = cpu.read_x_i32(instruction.rs1.value())?;
            cpu.write_x_i32(instruction.rd.value(), rs1 & imm)?;
            Ok(())
        }
    },
    Instruction{
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: (FUNC3_ORI as u32) << FUNC3_POS | 0b0010011,
        name: "ORI",
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let imm = sign_extend_12bit_to_32bit(instruction.imm);
            let rs1 = cpu.read_x_i32(instruction.rs1.value())?;
            cpu.write_x_i32(instruction.rd.value(), rs1 | imm)?;
            Ok(())
        }
    },
    Instruction{
        mask: OPCODE_MASK | FUNC3_MASK,
        bits: (FUNC3_XORI as u32) << FUNC3_POS | 0b0010011,
        name: "XORI",
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let imm = sign_extend_12bit_to_32bit(instruction.imm);
            let rs1 = cpu.read_x_i32(instruction.rs1.value())?;
            cpu.write_x_i32(instruction.rd.value(), rs1 ^ imm)?;
            Ok(())
        }
    },
    Instruction{
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0000000 << FUNC7_POS | 0b001 << FUNC3_POS | 0b0010011,
        name: "SLLI",
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let shamt = (instruction.imm & 0b11111) as u32;
            let res: u32 = cpu.read_x_u32(instruction.rs1.value())? << shamt;
            cpu.write_x_u32(instruction.rd.value(), res)?;
            Ok(())
        }
    },
    Instruction{
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0000000 << FUNC7_POS | 0b101 << FUNC3_POS | 0b0010011,
        name: "SRLI",
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let shamt = (instruction.imm & 0b11111) as u32;
            let res: u32 = cpu.read_x_u32(instruction.rs1.value())? >> shamt;
            cpu.write_x_u32(instruction.rd.value(), res)?;
            Ok(())
        }
    },
    Instruction{
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0100000 << FUNC7_POS | 0b101 << FUNC3_POS | 0b0010011,
        name: "SRAI",
        operation: |cpu, word| {
            let instruction = parse_instruction_i(word);
            let shamt = (instruction.imm & 0b11111) as u32;
            let res: i32 = cpu.read_x_i32(instruction.rs1.value())? >> shamt;
            cpu.write_x_i32(instruction.rd.value(), res)?;
            Ok(())
        }
    },
    Instruction{
        mask: OPCODE_MASK,
        bits: 0b0110111,
        name: "LUI",
        operation: |cpu, word| {
            let instruction = parse_instruction_u(word);
            cpu.write_x_u32(instruction.rd.value(), instruction.imm)?;
            Ok(())
        }
    },
    Instruction{
        mask: OPCODE_MASK,
        bits: 0b0010111,
        name: "AUIPC",
        operation: |cpu, word| {
            let instruction = parse_instruction_u(word);
            let res: u32 = instruction.imm.wrapping_add(cpu.read_pc_u32());

            cpu.write_pc_u32(res);
            cpu.set_skip_pc_increment_flag(); // Disable default pc increment logic
    
            cpu.write_x_u32(instruction.rd.value(), res)?;
            Ok(())
        }
    }
];