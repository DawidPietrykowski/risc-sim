use crate::isa::cpu::{Cpu, Operation};
use crate::isa::types::*;

use anyhow::{Ok, Result};

pub struct Add{
    instruction: RInstruction
}

impl Operation<RInstruction> for Add {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let rs1 = cpu.read_x_i32(self.instruction.rs1)?;
        let rs2 = cpu.read_x_i32(self.instruction.rs2)?;
        let (res, _) = rs1.overflowing_add(rs2);
        cpu.write_x_i32(self.instruction.rd, res)?;
        Ok(())
    }
    
    fn new(instruction: RInstruction) -> Self {
        Add { instruction: instruction }
    }
    
    fn instruction(&self) -> &RInstruction {
        &self.instruction
    }
}

pub struct Sub{
    instruction: RInstruction
}

impl Operation<RInstruction> for Sub {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let rs1 = cpu.read_x_i32(self.instruction.rs1)?;
        let rs2 = cpu.read_x_i32(self.instruction.rs2)?;
        let (res, _) = rs1.overflowing_sub(rs2);
        cpu.write_x_i32(self.instruction.rd, res)?;
        Ok(())
    }
    
    fn new(instruction: RInstruction) -> Self {
        Sub { instruction: instruction }
    }
    
    fn instruction(&self) -> &RInstruction {
        &self.instruction
    }
}

pub struct SLT{
    instruction: RInstruction
}

impl Operation<RInstruction> for SLT {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let rs1 = cpu.read_x_i32(self.instruction.rs1)?;
        let rs2 = cpu.read_x_i32(self.instruction.rs2)?;
        cpu.write_x_i32(self.instruction.rd, if rs1 < rs2 {1} else {0})?;
        Ok(())
    }
    
    fn new(instruction: RInstruction) -> Self {
        SLT { instruction: instruction }
    }
    
    fn instruction(&self) -> &RInstruction {
        &self.instruction
    }
}

pub struct SLTU{
    instruction: RInstruction
}

impl Operation<RInstruction> for SLTU {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let rs1 = cpu.read_x_u32(self.instruction.rs1)?;
        let rs2 = cpu.read_x_u32(self.instruction.rs2)?;
        cpu.write_x_i32(self.instruction.rd, if rs1 < rs2 {1} else {0})?;
        Ok(())
    }
    
    fn new(instruction: RInstruction) -> Self {
        SLTU { instruction: instruction }
    }
    
    fn instruction(&self) -> &RInstruction {
        &self.instruction
    }
}

pub struct AND{
    instruction: RInstruction
}

impl Operation<RInstruction> for AND {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let rs1 = cpu.read_x_u32(self.instruction.rs1)?;
        let rs2 = cpu.read_x_u32(self.instruction.rs2)?;
        cpu.write_x_u32(self.instruction.rd, rs1 & rs2)?;
        Ok(())
    }
    
    fn new(instruction: RInstruction) -> Self {
        AND { instruction: instruction }
    }
    
    fn instruction(&self) -> &RInstruction {
        &self.instruction
    }
}

pub struct OR {
    instruction: RInstruction
}

impl Operation<RInstruction> for OR {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let rs1 = cpu.read_x_u32(self.instruction.rs1)?;
        let rs2 = cpu.read_x_u32(self.instruction.rs2)?;
        cpu.write_x_u32(self.instruction.rd, rs1 | rs2)?;
        Ok(())
    }
    
    fn new(instruction: RInstruction) -> Self {
        OR { instruction: instruction }
    }
    
    fn instruction(&self) -> &RInstruction {
        &self.instruction
    }
}
pub struct XOR {
    instruction: RInstruction
}

impl Operation<RInstruction> for XOR {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let rs1 = cpu.read_x_u32(self.instruction.rs1)?;
        let rs2 = cpu.read_x_u32(self.instruction.rs2)?;
        cpu.write_x_u32(self.instruction.rd, rs1 ^ rs2)?;
        Ok(())
    }
    
    fn new(instruction: RInstruction) -> Self {
        XOR { instruction: instruction }
    }
    
    fn instruction(&self) -> &RInstruction {
        &self.instruction
    }
}


pub struct SLL {
    instruction: RInstruction
}

impl Operation<RInstruction> for SLL {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let shamt = (cpu.read_x_u32(self.instruction.rs2)? & 0b11111) as u32;
        let res: u32 = cpu.read_x_u32(self.instruction.rs1)? << shamt;
        cpu.write_x_u32(self.instruction.rd, res)?;
        Ok(())
    }
    
    fn new(instruction: RInstruction) -> Self {
        SLL { instruction: instruction }
    }
    
    fn instruction(&self) -> &RInstruction {
        &self.instruction
    }
}

pub struct SRL {
    instruction: RInstruction
}

impl Operation<RInstruction> for SRL {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let shamt = (cpu.read_x_u32(self.instruction.rs2)? & 0b11111) as u32;
        let res: u32 = cpu.read_x_u32(self.instruction.rs1)? >> shamt;
        cpu.write_x_u32(self.instruction.rd, res)?;
        Ok(())
    }
    
    fn new(instruction: RInstruction) -> Self {
        SRL { instruction: instruction }
    }
    
    fn instruction(&self) -> &RInstruction {
        &self.instruction
    }
}


pub struct SRA {
    instruction: RInstruction
}

impl Operation<RInstruction> for SRA {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let shamt = (cpu.read_x_u32(self.instruction.rs2)? & 0b11111) as u32;
        let res: i32 = cpu.read_x_i32(self.instruction.rs1)? >> shamt;
        cpu.write_x_i32(self.instruction.rd, res)?;
        Ok(())
    }
    
    fn new(instruction: RInstruction) -> Self {
        SRA { instruction: instruction }
    }
    
    fn instruction(&self) -> &RInstruction {
        &self.instruction
    }
}
