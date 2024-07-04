use crate::isa::types::InstructionType;
use crate::utils::binary_utils::*;

use anyhow::{bail, Context, Ok, Result};

use super::types::{decode_program_line, ProgramLine, Word};

pub struct Cpu {
    reg_x32: [u32; 31],
    reg_pc: u32,
    skip_pc_increment: bool,
    #[allow(dead_code)]
    program: Vec<ProgramLine>,
    pub mem_map: Vec<u8>,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            reg_x32: [0x0; 31],
            reg_pc: 0x0,
            skip_pc_increment: false,
            program: vec![],
            mem_map: vec![0; 1024 * 1024 * 1024],
        }
    }

    pub fn run_cycle(&mut self) -> Result<()> {
        // Fetch
        let instruction = self.fetch_instruction()?;

        // Increase PC
        self.reg_pc += 4;

        // Execute
        self.execute_program_line(instruction)?;

        Ok(())
    }

    pub fn execute_program_line(&mut self, program_line: ProgramLine) -> Result<()> {
        (program_line.instruction.operation)(self, &program_line.word)
    }

    pub fn execute_word(&mut self, word: Word) -> Result<()> {
        let program_line = decode_program_line(word)?;
        (program_line.instruction.operation)(self, &program_line.word)
    }

    fn fetch_instruction(&self) -> Result<ProgramLine> {
        Ok(*self
            .program
            .get(self.read_pc_u32() as usize / 4)
            .context("No instruction at index")?)
    }

    pub fn read_mem_u32(&mut self, addr: u32) -> Result<u32> {
        Ok((self.read_mem_u16(addr)? as u32) | (self.read_mem_u16(addr + 2)? as u32) << 16)
    }

    pub fn read_mem_u16(&mut self, addr: u32) -> Result<u16> {
        Ok((self.read_mem_u8(addr)? as u16) | (self.read_mem_u8(addr + 1)? as u16) << 8)
    }

    pub fn read_mem_u8(&mut self, addr: u32) -> Result<u8> {
        Ok(*self
            .mem_map
            .get(addr as usize)
            .context(format!("Out of bounds memory access at {}", addr))?)
    }

    pub fn write_mem_u8(&mut self, addr: u32, value: u8) -> Result<()> {
        if addr as usize >= self.mem_map.len() {
            bail!(format!("Out of bounds memory access at {}", addr))
        }
        self.mem_map[addr as usize] = value;
        Ok(())
    }

    pub fn write_mem_u16(&mut self, addr: u32, value: u16) -> Result<()> {
        self.write_mem_u8(addr, value as u8)?;
        self.write_mem_u8(addr + 1, (value >> 8) as u8)?;
        Ok(())
    }

    pub fn write_mem_u32(&mut self, addr: u32, value: u32) -> Result<()> {
        self.write_mem_u16(addr, value as u16)?;
        self.write_mem_u16(addr + 2, (value >> 16) as u16)?;
        Ok(())
    }

    pub fn read_x_u32(&self, id: u8) -> Result<u32> {
        let value = self
            .reg_x32
            .get(id as usize)
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

        let reg_value = self
            .reg_x32
            .get_mut(id as usize)
            .context(format!("Register x{} does not exist", id))?;

        *reg_value = i32_to_u32(value);
        Ok(())
    }

    pub fn write_x_u32(&mut self, id: u8, value: u32) -> Result<()> {
        if id == 0 {
            return Ok(()); // x0 is hardwired to 0
        }

        let reg_value = self
            .reg_x32
            .get_mut(id as usize)
            .context(format!("Register x{} does not exist", id))?;

        *reg_value = value;
        Ok(())
    }

    pub fn read_pc_u32(&self) -> u32 {
        self.reg_pc
    }

    pub fn write_pc_u32(&mut self, val: u32) {
        self.reg_pc = val;
    }

    pub fn set_skip_pc_increment_flag(&mut self) {
        self.skip_pc_increment = true;
    }
}
