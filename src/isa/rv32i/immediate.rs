use crate::isa::cpu::{Cpu, Operation};
use crate::isa::types::*;
use crate::utils::binary_utils::*;

use anyhow::{Ok, Result};

pub struct AddI {
    instruction: IInstruction,
}

impl Operation<IInstruction> for AddI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let imm = sign_extend_12bit_to_32bit(self.instruction.imm);
        let rs1 = cpu.read_x_i32(self.instruction.rs1)?;
        let (res, _) = imm.overflowing_add(rs1);
        cpu.write_x_i32(self.instruction.rd, res)?;
        Ok(())
    }

    fn new(instruction: IInstruction) -> Self {
        AddI {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &IInstruction {
        &self.instruction
    }
}
pub struct SLTI {
    instruction: IInstruction,
}

impl Operation<IInstruction> for SLTI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let imm = sign_extend_12bit_to_32bit(self.instruction.imm);
        let rs1 = cpu.read_x_i32(self.instruction.rs1)?;
        cpu.write_x_i32(self.instruction.rd, if rs1 < imm { 1 } else { 0 })?;
        Ok(())
    }

    fn new(instruction: IInstruction) -> Self {
        SLTI {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &IInstruction {
        &self.instruction
    }
}

pub struct ANDI {
    instruction: IInstruction,
}

impl Operation<IInstruction> for ANDI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let imm = sign_extend_12bit_to_32bit(self.instruction.imm);
        let rs1 = cpu.read_x_i32(self.instruction.rs1)?;
        cpu.write_x_i32(self.instruction.rd, rs1 & imm)?;
        Ok(())
    }

    fn new(instruction: IInstruction) -> Self {
        ANDI {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &IInstruction {
        &self.instruction
    }
}

pub struct ORI {
    instruction: IInstruction,
}

impl Operation<IInstruction> for ORI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let imm = sign_extend_12bit_to_32bit(self.instruction.imm);
        let rs1 = cpu.read_x_i32(self.instruction.rs1)?;
        cpu.write_x_i32(self.instruction.rd, rs1 | imm)?;
        Ok(())
    }

    fn new(instruction: IInstruction) -> Self {
        ORI {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &IInstruction {
        &self.instruction
    }
}
pub struct XORI {
    instruction: IInstruction,
}

impl Operation<IInstruction> for XORI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let imm = sign_extend_12bit_to_32bit(self.instruction.imm);
        let rs1 = cpu.read_x_i32(self.instruction.rs1)?;
        cpu.write_x_i32(self.instruction.rd, rs1 ^ imm)?;
        Ok(())
    }

    fn new(instruction: IInstruction) -> Self {
        XORI {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &IInstruction {
        &self.instruction
    }
}

pub struct SLLI {
    instruction: IInstruction,
}

impl Operation<IInstruction> for SLLI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let shamt = (self.instruction.imm & 0b11111) as u32;
        let res: u32 = cpu.read_x_u32(self.instruction.rs1)? << shamt;
        cpu.write_x_u32(self.instruction.rd, res)?;
        Ok(())
    }

    fn new(instruction: IInstruction) -> Self {
        SLLI {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &IInstruction {
        &self.instruction
    }
}

pub struct SRLI {
    instruction: IInstruction,
}

impl Operation<IInstruction> for SRLI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let shamt = (self.instruction.imm & 0b11111) as u32;
        let res: u32 = cpu.read_x_u32(self.instruction.rs1)? >> shamt;
        cpu.write_x_u32(self.instruction.rd, res)?;
        Ok(())
    }

    fn new(instruction: IInstruction) -> Self {
        SRLI {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &IInstruction {
        &self.instruction
    }
}

pub struct SRAI {
    instruction: IInstruction,
}

impl Operation<IInstruction> for SRAI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let shamt = (self.instruction.imm & 0b11111) as u32;
        let res: i32 = cpu.read_x_i32(self.instruction.rs1)? >> shamt;
        cpu.write_x_i32(self.instruction.rd, res)?;
        Ok(())
    }

    fn new(instruction: IInstruction) -> Self {
        SRAI {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &IInstruction {
        &self.instruction
    }
}

pub struct LUI {
    instruction: UInstruction,
}

impl Operation<UInstruction> for LUI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let shifted_imm = self.instruction.imm << 12;
        cpu.write_x_u32(self.instruction.rd, shifted_imm)?;
        Ok(())
    }

    fn new(instruction: UInstruction) -> Self {
        LUI {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &UInstruction {
        &self.instruction
    }
}

pub struct AUIPC {
    instruction: UInstruction,
}

impl Operation<UInstruction> for AUIPC {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let res: u32 = (self.instruction.imm << 12).wrapping_add(cpu.read_pc_u32());

        cpu.write_pc_u32(res);
        cpu.set_skip_pc_increment_flag(); // Disable default pc increment logic

        cpu.write_x_u32(self.instruction.rd, res)?;
        Ok(())
    }

    fn new(instruction: UInstruction) -> Self {
        AUIPC {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &UInstruction {
        &self.instruction
    }
}
