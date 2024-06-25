use crate::utils::binary_utils::*;

use anyhow::{Ok, Result, Context};

pub struct Cpu{
    reg_x32: [u32; 31],
    reg_pc: u32,
    skip_pc_increment: bool
}

impl Cpu { 
    pub fn new() -> Cpu {
        Cpu { 
            reg_x32: [0x0; 31],
            reg_pc: 0x0,
            skip_pc_increment: false
        }
    }  

    pub fn execute_operation<I>(&mut self, operation: &impl Operation<I>) -> Result<()> {
        self.skip_pc_increment = false;

        operation.execute(self)?;
        
        if !self.skip_pc_increment {
            self.reg_pc += 1;
        }
        Ok(())
    }

    pub fn read_x_u32(&self, id: u8) -> Result<u32> {
        let value = self.reg_x32.get(id as usize)
        .context(format!("Register x{} does not exist", id))?;
        
        Ok(*value)
    }

    pub fn read_x_i32(&self, id: u8) -> Result<i32> {
        Ok(u32_to_i32(self.read_x_u32(id)?))
    }

    pub fn write_x_i32(&mut self, id: u8, value: i32) -> Result<()> {
        if id == 0 {
            return Ok(()); // x0 is hardwired to 0
        }
        
        let reg_value = self.reg_x32.get_mut(id as usize)
        .context(format!("Register x{} does not exist", id))?;

        *reg_value = i32_to_u32(value);
        Ok(())
    }

    pub fn write_x_u32(&mut self, id: u8, value: u32) -> Result<()> {
        if id == 0 {
            return Ok(()); // x0 is hardwired to 0
        }

        let reg_value = self.reg_x32.get_mut(id as usize)
        .context(format!("Register x{} does not exist", id))?;

        *reg_value = value;
        Ok(())
    }

    pub fn read_pc_u32(&self) -> u32{
        self.reg_pc
    }

    pub fn write_pc_u32(&mut self, val: u32) {
        self.reg_pc = val;
    }

    pub fn set_skip_pc_increment_flag(&mut self) {
        self.skip_pc_increment = true;
    }
}

pub trait Operation<I>{
    fn new(instruction: I) -> Self;
    fn instruction(&self) -> &I;
    
    fn execute(&self, cpu: &mut Cpu) -> Result<()>;
}
