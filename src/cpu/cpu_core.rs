use std::fmt::Display;

use crate::{
    elf::elf_loader::{load_program_to_memory, ElfFile},
    system::{kernel::Kernel, passthrough_kernel::PassthroughKernel},
    types::ABIRegister,
    utils::binary_utils::*,
};

use super::memory::{memory_core::Memory, program_cache::ProgramCache, vec_memory::VecMemory};
use crate::types::{decode_program_line, ProgramLine, Word};
use anyhow::{bail, Context, Result};

pub struct Cpu {
    reg_x32: [u32; 32],
    reg_pc: u32,
    pub current_instruction_pc: u32,
    pub memory: Box<dyn Memory>,
    program_cache: ProgramCache,
    program_memory_offset: u32,
    halted: bool,
    pub program_brk: u32,
    #[cfg(not(feature = "maxperf"))]
    pub debug_enabled: bool,
    pub kernel: Box<dyn Kernel>,
}

impl Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Registers:")?;
        for (i, reg) in self.reg_x32.iter().filter(|reg| *reg != &0).enumerate() {
            writeln!(f, "x{}: {:#010x}", i + 1, reg)?;
        }
        writeln!(f, "PC: {:#010x}", self.reg_pc)?;
        writeln!(f, "Program Break: {:#010x}", self.program_brk)?;
        writeln!(f, "Halted: {}", self.halted)?;
        writeln!(
            f,
            "Program Memory Offset: {:#010x}",
            self.program_memory_offset
        )?;
        writeln!(f, "Memory: {:?}", self.memory)
    }
}

const INITIAL_STACK_POINTER: u32 = 0xbfffff00; // TODO: Calculate during program load

impl Default for Cpu {
    fn default() -> Self {
        Cpu {
            reg_x32: [0x0; 32],
            reg_pc: 0x0,
            current_instruction_pc: 0x0,
            memory: Box::new(VecMemory::new()),
            program_cache: ProgramCache::empty(),
            program_memory_offset: 0x0,
            halted: false,
            program_brk: 0,
            #[cfg(not(feature = "maxperf"))]
            debug_enabled: false,
            kernel: Box::<PassthroughKernel>::default(),
        }
    }
}

impl Cpu {
    pub fn new<M, K>(memory: M, kernel: K) -> Cpu
    where
        M: Memory + 'static,
        K: Kernel + 'static,
    {
        Cpu {
            reg_x32: [0x0; 32],
            reg_pc: 0x0,
            current_instruction_pc: 0x0,
            memory: Box::new(memory),
            program_cache: ProgramCache::empty(),
            program_memory_offset: 0x0,
            halted: false,
            program_brk: 0,
            #[cfg(not(feature = "maxperf"))]
            debug_enabled: false,
            kernel: Box::new(kernel),
        }
    }

    // TODO: Refactor such that this logic is done by the kernel
    pub fn load_program_from_elf(&mut self, elf: ElfFile) -> Result<()> {
        let program_file = load_program_to_memory(elf, self.memory.as_mut())?;
        self.reg_pc = program_file.entry_point;
        self.program_cache = ProgramCache::new(
            program_file.program_memory_offset,
            program_file.program_memory_offset + program_file.program_size,
            self.memory.as_mut(),
        )
        .unwrap();
        self.write_x_u32(ABIRegister::SP.to_x_reg_id() as u8, INITIAL_STACK_POINTER)
            .unwrap();
        self.program_brk = program_file.end_of_data_addr;
        Ok(())
    }

    pub fn load_program_from_opcodes(&mut self, opcodes: Vec<u32>, entry_point: u32) -> Result<()> {
        let program_size = opcodes.len() as u32 * 4;

        for (id, val) in opcodes.iter().enumerate() {
            self.memory
                .write_mem_u32(entry_point + 4u32 * (id as u32), *val)
                .unwrap();
        }

        self.reg_pc = entry_point;

        self.program_cache = ProgramCache::new(
            entry_point,
            entry_point + program_size,
            self.memory.as_mut(),
        )
        .unwrap();

        self.program_brk = entry_point + program_size;
        Ok(())
    }

    pub fn run_cycle(&mut self) -> Result<()> {
        // Check if CPU is halted
        if self.halted {
            bail!("CPU is halted");
        }

        // Fetch
        let instruction = self.fetch_instruction()?;

        #[cfg(not(feature = "maxperf"))]
        if self.debug_enabled {
            println!("\nPC({:#x}) {}", self.reg_pc, instruction);
        }

        // Increase PC
        self.current_instruction_pc = self.reg_pc;
        self.reg_pc += 4;

        // Execute
        self.execute_program_line(&instruction)?;

        Ok(())
    }

    #[cfg(feature = "maxperf")]
    pub fn run_cycle_uncheked(&mut self) -> Result<()> {
        // Check if CPU is halted
        if self.halted {
            bail!("CPU is halted");
        }

        // Fetch
        let instruction = self.fetch_instruction_unchecked();

        // Increase PC
        self.current_instruction_pc = self.reg_pc;
        self.reg_pc += 4;

        // Execute
        let _ = self.execute_program_line(&instruction);

        Ok(())
    }

    #[inline(always)]
    pub fn execute_program_line(&mut self, program_line: &ProgramLine) -> Result<()> {
        (program_line.instruction.operation)(self, &program_line.word)
    }

    pub fn execute_word(&mut self, word: Word) -> Result<()> {
        let program_line = decode_program_line(word)?;
        (program_line.instruction.operation)(self, &program_line.word)
    }

    pub fn set_halted(&mut self) {
        self.halted = true;
    }

    #[cfg(not(feature = "maxperf"))]
    pub fn set_debug_enabled(&mut self, debug_enabled: bool) {
        self.debug_enabled = debug_enabled;
    }

    #[allow(unused)]
    pub fn debug_print<F>(&self, message: F)
    where
        F: FnOnce() -> String,
    {
        #[cfg(not(feature = "maxperf"))]
        if self.debug_enabled {
            println!("{}", message());
        }
    }

    fn fetch_instruction(&mut self) -> Result<ProgramLine> {
        if let Some(cache_line) = self.program_cache.try_get_line(self.reg_pc) {
            Ok(cache_line)
        } else {
            decode_program_line(Word(
                self.memory
                    .read_mem_u32(self.read_pc_u32())
                    .context("No instruction at pc")?,
            ))
        }
    }

    #[cfg(feature = "maxperf")]
    fn fetch_instruction_unchecked(&self) -> ProgramLine {
        self.program_cache.get_line_unchecked(self.reg_pc)
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
            // .context(format!("Register x{} does not exist", id))?;
            .context("Register does not exist")?;

        Ok(*value)
    }

    pub fn read_x_i32(&self, id: u8) -> Result<i32> {
        Ok(u32_to_i32(self.read_x_u32(id)?))
    }

    pub fn write_x_i32(&mut self, id: u8, value: i32) -> Result<()> {
        if id == 0 {
            return Ok(()); // x0 is hardwired to 0
        }

        #[cfg(not(feature = "maxperf"))]
        let reg_value = self
            .reg_x32
            .get_mut(id as usize)
            // .context(format!("Register x{} does not exist", id))?;
            .context("Register does not exist")?;
        #[cfg(feature = "maxperf")]
        let reg_value = unsafe { self.reg_x32.get_unchecked_mut(id as usize) }; // SAFETY: For properly compiled code 0 <= id < 32

        *reg_value = i32_to_u32(value);
        Ok(())
    }

    pub fn write_x_u32(&mut self, id: u8, value: u32) -> Result<()> {
        if id == 0 {
            return Ok(()); // x0 is hardwired to 0
        }

        #[cfg(not(feature = "maxperf"))]
        let reg_value = self
            .reg_x32
            .get_mut(id as usize)
            // .context(format!("Register x{} does not exist", id))?;
            .context("Register does not exist")?;
        #[cfg(feature = "maxperf")]
        let reg_value = unsafe { self.reg_x32.get_unchecked_mut(id as usize) }; // SAFETY: For properly compiled code 0 <= id < 32

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

    pub fn read_buf(&mut self, addr: u32, buf: &mut [u8]) -> Result<()> {
        self.memory.read_buf(addr, buf)
    }

    pub fn write_buf(&mut self, addr: u32, buf: &[u8]) -> Result<()> {
        self.memory.write_buf(addr, buf)
    }

    pub fn read_c_string(&mut self, addr: u32) -> Result<String> {
        let mut result = String::new();
        let mut current_addr = addr;
        loop {
            let byte = self.read_mem_u8(current_addr)?;
            if byte == 0 {
                break;
            }
            result.push(byte as char);
            current_addr += 1;
        }
        Ok(result)
    }
}
