use std::collections::HashMap;

use crate::isa::types::InstructionType;
use crate::utils::binary_utils::*;

use anyhow::{Context, Ok, Result};

use super::types::{decode_program_line, InstructionData, ProgramLine, Word};

pub struct Cpu {
    reg_x32: [u32; 31],
    reg_pc: u32,
    skip_pc_increment: bool,
    #[allow(dead_code)]
    program: Vec<ProgramLine>,
    mem_map: HashMap<u32, u32>,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            reg_x32: [0x0; 31],
            reg_pc: 0x0,
            skip_pc_increment: false,
            program: vec![],
            mem_map: HashMap::new(),
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

    pub fn execute_operation<I: InstructionType>(
        &mut self,
        operation: &impl Operation<I>,
    ) -> Result<()> {
        // self.skip_pc_increment = false;

        self.reg_pc += 4;

        operation.execute(self)?;

        // if !self.skip_pc_increment {
        // self.reg_pc += 1;
        // }
        Ok(())
    }

    pub fn execute_instruction(&mut self, instruction: InstructionData) -> Result<()> {
        self.reg_pc += 4;

        let operation = match instruction {
            InstructionData::R(ins) => {}
            InstructionData::I(ins) => todo!(),
            InstructionData::S(ins) => todo!(),
            InstructionData::SB(ins) => todo!(),
            InstructionData::U(ins) => todo!(),
            InstructionData::UJ(ins) => todo!(),
        };

        Ok(())
    }

    pub fn read_mem_u32(&mut self, addr: u32) -> Result<u32> {
        // TODO: optimize
        Ok(*self.mem_map.clone().get(&addr).unwrap_or_else(|| {
            self.mem_map.insert(addr, 0);
            &0
        }))
    }

    pub fn write_mem_u32(&mut self, addr: u32, value: u32) {
        self.mem_map.insert(addr, value);
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

pub trait Operation<I: InstructionType> {
    fn new(instruction: I) -> Self;
    fn instruction(&self) -> &I;
    fn execute(&self, cpu: &mut Cpu) -> Result<()>;
}
