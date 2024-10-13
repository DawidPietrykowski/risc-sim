use std::fmt::Display;

use crate::{
    elf::elf_loader::{load_program_to_memory, ElfFile},
    isa::
        csr::csr_types::{CSRAddress, CSRTable, MisaCSR},
    system::{kernel::Kernel, passthrough_kernel::PassthroughKernel},
    types::ABIRegister,
    utils::binary_utils::*,
};

use super::memory::{
    memory_core::Memory, mmu::walk_page_table_sv39, program_cache::ProgramCache,
    raw_vec_memory::RawVecMemory,
};
use crate::types::{decode_program_line, ProgramLine, Word};
use anyhow::{bail, Context, Result};

#[derive(PartialEq, Clone, Copy)]
pub enum CpuMode {
    RV32,
    RV64,
}

#[derive(PartialEq, Clone, Copy)]
pub enum PrivilegeMode {
    User = 0,
    Supervisor = 1,
    Machine = 3,
}

pub struct Cpu {
    reg_x32: [u32; 32],
    reg_x64: [u64; 32],
    reg_pc: u32,
    reg_pc_64: u64,
    pub current_instruction_pc: u32,
    pub current_instruction_pc_64: u64,
    pub memory: Box<dyn Memory>,
    program_cache: ProgramCache,
    program_memory_offset: u64,
    halted: bool,
    pub program_brk: u64,
    #[cfg(not(feature = "maxperf"))]
    pub debug_enabled: bool,
    pub kernel: Box<dyn Kernel>,
    pub csr_table: CSRTable,
    pub arch_mode: CpuMode,
    pub simulate_kernel: bool,
    pub privilege_mode: PrivilegeMode,
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

const INITIAL_STACK_POINTER_32: u32 = 0xbfffff00; // TODO: Calculate during program load
const INITIAL_STACK_POINTER_64: u64 = 0x00007FFFFFFFFFFF; // TODO: Calculate during program load

impl Default for Cpu {
    fn default() -> Self {
        let mut cpu = Cpu {
            reg_x32: [0x0; 32],
            reg_x64: [0x0; 32],
            reg_pc: 0x0,
            reg_pc_64: 0x0,
            current_instruction_pc: 0x0,
            current_instruction_pc_64: 0x0,
            memory: Box::new(RawVecMemory::new()),
            program_cache: ProgramCache::empty(),
            program_memory_offset: 0x0,
            halted: false,
            program_brk: 0,
            #[cfg(not(feature = "maxperf"))]
            debug_enabled: false,
            kernel: Box::<PassthroughKernel>::default(),
            csr_table: CSRTable::new(),
            arch_mode: CpuMode::RV32,
            simulate_kernel: true,
            privilege_mode: PrivilegeMode::Machine,
        };
        cpu.setup_csrs();
        cpu
    }
}

impl Cpu {
    pub fn new<M, K>(memory: M, kernel: K, mode: CpuMode) -> Cpu
    where
        M: Memory + 'static,
        K: Kernel + 'static,
    {
        let mut cpu = Cpu {
            reg_x32: [0x0; 32],
            reg_x64: [0x0; 32],
            reg_pc: 0x0,
            reg_pc_64: 0x0,
            current_instruction_pc: 0x0,
            current_instruction_pc_64: 0x0,
            memory: Box::new(memory),
            program_cache: ProgramCache::empty(),
            program_memory_offset: 0x0,
            halted: false,
            program_brk: 0,
            #[cfg(not(feature = "maxperf"))]
            debug_enabled: false,
            kernel: Box::new(kernel),
            csr_table: CSRTable::new(),
            arch_mode: mode,
            simulate_kernel: true,
            privilege_mode: PrivilegeMode::Machine,
        };
        cpu.setup_csrs();
        cpu
    }

    // TODO: Refactor such that this logic is done by the kernel
    pub fn load_program_from_elf(&mut self, elf: ElfFile) -> Result<()> {
        let program_file = load_program_to_memory(elf, self.memory.as_mut(), self.arch_mode)?;

        if self.arch_mode == CpuMode::RV64 {
            self.reg_pc_64 = program_file.entry_point;
        } else {
            self.reg_pc = program_file.entry_point as u32;
        }

        self.program_cache = ProgramCache::new(
            program_file.program_memory_offset,
            program_file.program_memory_offset + program_file.program_size,
            self.memory.as_mut(),
            self.arch_mode,
        )
        .unwrap();
        if self.arch_mode == CpuMode::RV64 {
            self.write_x_u64(
                ABIRegister::SP.to_x_reg_id() as u8,
                INITIAL_STACK_POINTER_64,
            )
            .unwrap();
        } else {
            self.write_x_u32(
                ABIRegister::SP.to_x_reg_id() as u8,
                INITIAL_STACK_POINTER_32,
            )
            .unwrap();
        }
        self.program_brk = program_file.end_of_data_addr;

        self.setup_csrs();

        Ok(())
    }

    pub fn load_program_from_opcodes(
        &mut self,
        opcodes: Vec<u32>,
        entry_point: u64,
        mode: CpuMode,
    ) -> Result<()> {
        let program_size = opcodes.len() as u64 * 4;

        for (id, val) in opcodes.iter().enumerate() {
            self.memory
                .write_mem_u32(entry_point + 4u64 * (id as u64), *val)
                .unwrap();
        }

        if self.arch_mode == CpuMode::RV64 {
            self.reg_pc_64 = entry_point;
        } else {
            self.reg_pc = entry_point as u32;
        }

        self.program_cache = ProgramCache::new(
            entry_point,
            entry_point + program_size,
            self.memory.as_mut(),
            mode,
        )
        .unwrap();

        self.program_brk = entry_point + program_size;
        Ok(())
    }

    fn setup_csrs(&mut self) {
        let mut misa = MisaCSR(0);
        misa.set_extension_i(true);
        misa.set_extension_m(true);
        match self.arch_mode {
            CpuMode::RV32 => {
                misa.set_mxl_32(1);
                self.csr_table
                    .write32(CSRAddress::Misa.as_u12(), misa.0 as u32);
            }
            CpuMode::RV64 => {
                misa.set_mxl_64(2);
                self.csr_table.write64(CSRAddress::Misa.as_u12(), misa.0);
            }
        }
        self.csr_table.write32(CSRAddress::Mvendorid.as_u12(), 0);
        self.csr_table
            .write_xlen(CSRAddress::Mhartid.as_u12(), 0, self.arch_mode);
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
        if self.arch_mode == CpuMode::RV64 {
            self.current_instruction_pc_64 = self.reg_pc_64;
            self.reg_pc_64 += 4;
        } else {
            self.current_instruction_pc = self.reg_pc;
            self.reg_pc += 4;
        }

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
        if self.arch_mode == CpuMode::RV64 {
            self.current_instruction_pc_64 = self.reg_pc_64;
            self.reg_pc_64 += 4;
        } else {
            self.current_instruction_pc = self.reg_pc;
            self.reg_pc += 4;
        }

        // Execute
        let _ = self.execute_program_line(&instruction);

        Ok(())
    }

    #[inline(always)]
    pub fn execute_program_line(&mut self, program_line: &ProgramLine) -> Result<()> {
        (program_line.instruction.operation)(self, &program_line.word)
    }

    pub fn execute_word(&mut self, word: Word) -> Result<()> {
        let program_line = decode_program_line(word, self.arch_mode)?;
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

    pub fn read_pc(&self) -> u64 {
        if self.arch_mode == CpuMode::RV32 {
            self.reg_pc as u64
        } else {
            self.reg_pc_64
        }
    }

    fn fetch_instruction(&mut self) -> Result<ProgramLine> {
        let pc = self.read_pc();
        if let Some(cache_line) = self.program_cache.try_get_line(pc) {
            Ok(cache_line)
        } else {
            decode_program_line(
                Word(
                    self.memory
                        .read_mem_u32(pc)
                        .context("No instruction at pc")?,
                ),
                self.arch_mode,
            )
        }
    }

    #[cfg(feature = "maxperf")]
    fn fetch_instruction_unchecked(&self) -> ProgramLine {
        self.program_cache.get_line_unchecked(self.read_pc())
    }

    fn translate_address_if_needed(&mut self, addr: u64) -> Result<u64> {
        let satp = self.csr_table.read64(CSRAddress::Satp.as_u12());
        if satp != 0 {
            walk_page_table_sv39(addr, satp, self)
        } else {
            Ok(addr)
        }
    }

    pub fn read_mem_u64(&mut self, addr: u64) -> Result<u64> {
        let addr = self.translate_address_if_needed(addr)?;
        self.memory.read_mem_u64(addr)
    }

    pub fn read_mem_u32(&mut self, addr: u64) -> Result<u32> {
        let addr = self.translate_address_if_needed(addr)?;
        self.memory.read_mem_u32(addr)
    }

    pub fn read_mem_u16(&mut self, addr: u64) -> Result<u16> {
        let addr = self.translate_address_if_needed(addr)?;
        self.memory.read_mem_u16(addr)
    }

    pub fn read_mem_u8(&mut self, addr: u64) -> Result<u8> {
        let addr = self.translate_address_if_needed(addr)?;
        self.memory.read_mem_u8(addr)
    }

    pub fn write_mem_u8(&mut self, addr: u64, value: u8) -> Result<()> {
        let addr = self.translate_address_if_needed(addr)?;
        self.memory.write_mem_u8(addr, value)
    }

    pub fn write_mem_u16(&mut self, addr: u64, value: u16) -> Result<()> {
        let addr = self.translate_address_if_needed(addr)?;
        self.memory.write_mem_u16(addr, value)
    }

    pub fn write_mem_u32(&mut self, addr: u64, value: u32) -> Result<()> {
        let addr = self.translate_address_if_needed(addr)?;
        self.memory.write_mem_u32(addr, value)
    }

    pub fn write_mem_u64(&mut self, addr: u64, value: u64) -> Result<()> {
        let addr = self.translate_address_if_needed(addr)?;
        self.memory.write_mem_u64(addr, value)
    }

    pub fn read_x_u32(&self, id: u8) -> Result<u32> {
        #[cfg(feature = "maxperf")]
        {
            unsafe { return Ok(*self.reg_x32.get_unchecked(id as usize)) }
        }
        #[cfg(not(feature = "maxperf"))]
        {
            let value = self
                .reg_x32
                .get(id as usize)
                .context(format!("Register x{} does not exist", id))?;

            return Ok(*value);
        }
    }

    pub fn read_x_u64(&self, id: u8) -> Result<u64> {
        #[cfg(feature = "maxperf")]
        {
            unsafe { return Ok(*self.reg_x64.get_unchecked(id as usize)) }
        }
        #[cfg(not(feature = "maxperf"))]
        {
            let value = self
                .reg_x64
                .get(id as usize)
                .context(format!("Register x{} does not exist", id))?;

            return Ok(*value);
        }
    }

    pub fn read_x_i32(&self, id: u8) -> Result<i32> {
        Ok(u32_to_i32(self.read_x_u32(id)?))
    }

    pub fn read_x_i64(&self, id: u8) -> Result<i64> {
        Ok(u64_to_i64(self.read_x_u64(id)?))
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

    pub fn write_x_i64(&mut self, id: u8, value: i64) -> Result<()> {
        if id == 0 {
            return Ok(()); // x0 is hardwired to 0
        }

        #[cfg(not(feature = "maxperf"))]
        let reg_value = self
            .reg_x64
            .get_mut(id as usize)
            // .context(format!("Register x{} does not exist", id))?;
            .context("Register does not exist")?;
        #[cfg(feature = "maxperf")]
        let reg_value = unsafe { self.reg_x64.get_unchecked_mut(id as usize) }; // SAFETY: For properly compiled code 0 <= id < 32

        *reg_value = i64_to_u64(value);
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

    pub fn write_x_u64(&mut self, id: u8, value: u64) -> Result<()> {
        if id == 0 {
            return Ok(()); // x0 is hardwired to 0
        }

        #[cfg(not(feature = "maxperf"))]
        let reg_value = self
            .reg_x64
            .get_mut(id as usize)
            // .context(format!("Register x{} does not exist", id))?;
            .context("Register does not exist")?;
        #[cfg(feature = "maxperf")]
        let reg_value = unsafe { self.reg_x64.get_unchecked_mut(id as usize) }; // SAFETY: For properly compiled code 0 <= id < 32

        *reg_value = value;
        Ok(())
    }

    pub fn read_pc_u32(&self) -> u32 {
        self.reg_pc
    }

    pub fn read_pc_u64(&self) -> u64 {
        self.reg_pc_64
    }

    pub fn write_pc_u32(&mut self, val: u32) {
        self.reg_pc = val;
    }

    pub fn write_pc_u64(&mut self, val: u64) {
        self.reg_pc_64 = val;
    }

    pub fn read_current_instruction_addr_u32(&self) -> u32 {
        self.current_instruction_pc
    }

    pub fn read_current_instruction_addr_u64(&self) -> u64 {
        self.current_instruction_pc_64
    }

    pub fn read_buf(&mut self, addr: u64, buf: &mut [u8]) -> Result<()> {
        let addr = self.translate_address_if_needed(addr)?;
        self.memory.read_buf(addr, buf)
    }

    pub fn write_buf(&mut self, addr: u64, buf: &[u8]) -> Result<()> {
        let addr = self.translate_address_if_needed(addr)?;
        self.memory.write_buf(addr, buf)
    }

    pub fn read_c_string(&mut self, addr: u64) -> Result<String> {
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
