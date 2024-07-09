use std::fmt::Display;

use crate::utils::binary_utils::*;

use anyhow::{Context, Ok, Result};

use super::{memory::Memory, types::{decode_program_line, ProgramLine, Word}};

pub struct Cpu {
    reg_x32: [u32; 31],
    reg_pc: u32,
    pub current_instruction_pc: u32,
    skip_pc_increment: bool,
    program: Vec<ProgramLine>,
    memory: Memory,
    stdout_buffer: Vec<u8>,
    program_memory_offset: u32,
}

impl Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Registers:")?;
        for (i, reg) in self.reg_x32.iter().filter(|reg| *reg != &0).enumerate() {
            writeln!(f, "x{}: {:#010x}", i + 1, reg)?;
        }
        writeln!(f, "PC: {:#010x}", self.reg_pc)
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut stdout_buffer = Vec::<u8>::new();
        stdout_buffer.reserve(1024);
        Cpu {
            reg_x32: [0x0; 31],
            reg_pc: 0x0,
            current_instruction_pc: 0x0,
            skip_pc_increment: false,
            program: vec![],
            memory: Memory::new(),
            stdout_buffer: stdout_buffer,
            program_memory_offset: 0x0,
        }
    }

    pub fn load_program(&mut self, memory: Memory, entry_point: u32) {
        self.memory = memory;
        self.reg_pc = entry_point;
    }

    pub fn run_cycle(&mut self) -> Result<()> {
        print!("{}[{:#010x}] ", String::from_utf8(self.stdout_buffer.clone()).unwrap(), self.reg_pc);

        // Fetch
        let instruction = self.fetch_instruction()?;

        println!("{}", instruction);

        // Increase PC
        self.current_instruction_pc = self.reg_pc;
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
        decode_program_line(Word(*&self
                    .memory
            .read_mem_u32(self.read_pc_u32())
            .context(format!("No instruction at index {:#x}", self.read_pc_u32()))?))
    }

    pub fn read_mem_u32(&mut self, addr: u32) -> Result<u32> {
        self.memory.read_mem_u32(addr)
    }

    pub fn read_mem_u16(&mut self, addr: u32) -> Result<u16> {
        self.memory.read_mem_u16(addr)
    }

    pub fn read_mem_u8(&mut self, addr: u32) -> Result<u8> {
        self.memory.read_mem_u8(addr)
    }

    pub fn write_mem_u8(&mut self, addr: u32, value: u8) -> Result<()> {
        self.memory.write_mem_u8(addr, value)
    }

    pub fn write_mem_u16(&mut self, addr: u32, value: u16) -> Result<()> {
        self.memory.write_mem_u16(addr, value)
    }

    pub fn write_mem_u32(&mut self, addr: u32, value: u32) -> Result<()> {
        self.memory.write_mem_u32(addr, value)
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

    pub fn read_current_instruction_addr_u32(&self) -> u32 {
        self.current_instruction_pc
    }

    pub fn push_stdout(&mut self, value: u8) {
        self.stdout_buffer.push(value);
    }
}
