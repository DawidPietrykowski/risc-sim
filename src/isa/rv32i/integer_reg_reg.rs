use crate::isa::cpu::{Cpu, Operation};
use crate::isa::types::*;

use anyhow::{Ok, Result};

pub struct Add {
    instruction: RInstructionData,
}

impl Operation<RInstructionData> for Add {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let rs1 = cpu.read_x_i32(self.instruction.rs1)?;
        let rs2 = cpu.read_x_i32(self.instruction.rs2)?;
        let (res, _) = rs1.overflowing_add(rs2);
        cpu.write_x_i32(self.instruction.rd, res)?;
        Ok(())
    }

    fn new(instruction: RInstructionData) -> Self {
        Add {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &RInstructionData {
        &self.instruction
    }
}

pub struct Sub {
    instruction: RInstructionData,
}

impl Operation<RInstructionData> for Sub {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let rs1 = cpu.read_x_i32(self.instruction.rs1)?;
        let rs2 = cpu.read_x_i32(self.instruction.rs2)?;
        let (res, _) = rs1.overflowing_sub(rs2);
        cpu.write_x_i32(self.instruction.rd, res)?;
        Ok(())
    }

    fn new(instruction: RInstructionData) -> Self {
        Sub {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &RInstructionData {
        &self.instruction
    }
}

pub struct SLT {
    instruction: RInstructionData,
}

impl Operation<RInstructionData> for SLT {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let rs1 = cpu.read_x_i32(self.instruction.rs1)?;
        let rs2 = cpu.read_x_i32(self.instruction.rs2)?;
        cpu.write_x_i32(self.instruction.rd, if rs1 < rs2 { 1 } else { 0 })?;
        Ok(())
    }

    fn new(instruction: RInstructionData) -> Self {
        SLT {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &RInstructionData {
        &self.instruction
    }
}

pub struct SLTU {
    instruction: RInstructionData,
}

impl Operation<RInstructionData> for SLTU {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let rs1 = cpu.read_x_u32(self.instruction.rs1)?;
        let rs2 = cpu.read_x_u32(self.instruction.rs2)?;
        cpu.write_x_i32(self.instruction.rd, if rs1 < rs2 { 1 } else { 0 })?;
        Ok(())
    }

    fn new(instruction: RInstructionData) -> Self {
        SLTU {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &RInstructionData {
        &self.instruction
    }
}

pub struct AND {
    instruction: RInstructionData,
}

impl Operation<RInstructionData> for AND {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let rs1 = cpu.read_x_u32(self.instruction.rs1)?;
        let rs2 = cpu.read_x_u32(self.instruction.rs2)?;
        cpu.write_x_u32(self.instruction.rd, rs1 & rs2)?;
        Ok(())
    }

    fn new(instruction: RInstructionData) -> Self {
        AND {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &RInstructionData {
        &self.instruction
    }
}

pub struct OR {
    instruction: RInstructionData,
}

impl Operation<RInstructionData> for OR {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let rs1 = cpu.read_x_u32(self.instruction.rs1)?;
        let rs2 = cpu.read_x_u32(self.instruction.rs2)?;
        cpu.write_x_u32(self.instruction.rd, rs1 | rs2)?;
        Ok(())
    }

    fn new(instruction: RInstructionData) -> Self {
        OR {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &RInstructionData {
        &self.instruction
    }
}
pub struct XOR {
    instruction: RInstructionData,
}

impl Operation<RInstructionData> for XOR {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let rs1 = cpu.read_x_u32(self.instruction.rs1)?;
        let rs2 = cpu.read_x_u32(self.instruction.rs2)?;
        cpu.write_x_u32(self.instruction.rd, rs1 ^ rs2)?;
        Ok(())
    }

    fn new(instruction: RInstructionData) -> Self {
        XOR {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &RInstructionData {
        &self.instruction
    }
}

pub struct SLL {
    instruction: RInstructionData,
}

impl Operation<RInstructionData> for SLL {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let shamt = (cpu.read_x_u32(self.instruction.rs2)? & 0b11111) as u32;
        let res: u32 = cpu.read_x_u32(self.instruction.rs1)? << shamt;
        cpu.write_x_u32(self.instruction.rd, res)?;
        Ok(())
    }

    fn new(instruction: RInstructionData) -> Self {
        SLL {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &RInstructionData {
        &self.instruction
    }
}

pub struct SRL {
    instruction: RInstructionData,
}

impl Operation<RInstructionData> for SRL {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let shamt = (cpu.read_x_u32(self.instruction.rs2)? & 0b11111) as u32;
        let res: u32 = cpu.read_x_u32(self.instruction.rs1)? >> shamt;
        cpu.write_x_u32(self.instruction.rd, res)?;
        Ok(())
    }

    fn new(instruction: RInstructionData) -> Self {
        SRL {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &RInstructionData {
        &self.instruction
    }
}

pub struct SRA {
    instruction: RInstructionData,
}

impl Operation<RInstructionData> for SRA {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let shamt = (cpu.read_x_u32(self.instruction.rs2)? & 0b11111) as u32;
        let res: i32 = cpu.read_x_i32(self.instruction.rs1)? >> shamt;
        cpu.write_x_i32(self.instruction.rd, res)?;
        Ok(())
    }

    fn new(instruction: RInstructionData) -> Self {
        SRA {
            instruction: instruction,
        }
    }

    fn instruction(&self) -> &RInstructionData {
        &self.instruction
    }
}
