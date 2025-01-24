use std::{fmt::Display, fs::File};

use crate::{
    elf::elf_loader::{load_kernel_to_memory, load_program_to_memory, ElfFile},
    isa::{
        csr::csr_types::{CSRAddress, CSRTable},
        traps::check_pending_interrupts,
    },
    system::{
        kernel::Kernel,
        passthrough_kernel::PassthroughKernel,
        plic::{plic_check_pending, PLIC_ADDR},
        uart::UART_ADDR,
        virtio::{BlockDevice, VIRTIO_0_ADDR},
    },
    types::{decode_program_line_unchecked, ABIRegister, Instruction},
    utils::binary_utils::*,
};

use super::{
    memory::{
        memory_core::Memory,
        mmu::walk_page_table_sv39,
        program_cache::ProgramCache,
        raw_memory::ContinuousMemory,
        raw_vec_memory::RawVecMemory,
        user_memory::{UserMemory, HEAP_SIZE, STACK_SIZE},
    },
    memory_access::*,
};
use crate::{
    types::{decode_program_line, ProgramLine, Word},
    utils::data::CircularBuffer,
};
use anyhow::{bail, Ok, Result};

#[derive(PartialEq, Clone, Copy)]
pub enum CpuMode {
    RV32,
    RV64,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum ExecutionMode {
    UserSpace,
    Bare,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum PrivilegeMode {
    User = 0,
    Supervisor = 1,
    Machine = 3,
}

pub struct Peripherals {
    pub uart: ContinuousMemory,
    pub virtio: ContinuousMemory,
    pub plic: ContinuousMemory,
}

pub struct Cpu {
    reg_x32: [u32; 32],
    reg_x64: [u64; 32],
    reg_f: [f64; 32],
    reg_pc_64: u64,
    pub current_instruction_pc_64: u64,
    pub memory: Box<dyn Memory>,
    program_cache: ProgramCache,
    program_memory_offset: u64,
    halted: bool,
    pub program_brk: u64,
    pub kernel: Box<dyn Kernel>,
    pub csr_table: CSRTable,
    pub arch_mode: CpuMode,
    pub privilege_mode: PrivilegeMode,
    pub pc_history: CircularBuffer<(u64, Option<Instruction>, u64)>,
    pub block_device: Option<BlockDevice>,
    pub execution_mode: ExecutionMode,
    pub peripherals: Option<Peripherals>,
}

impl Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Registers:")?;
        if self.arch_mode == CpuMode::RV64 {
            for (i, reg) in self
                .reg_x64
                .iter()
                .enumerate()
                .filter(|(_, reg)| *reg != &0)
            {
                writeln!(f, "x{}: {:#010x}", i, reg)?;
            }
            writeln!(f, "PC: {:#010x}", self.reg_pc_64)?;
        } else {
            for (i, reg) in self
                .reg_x32
                .iter()
                .enumerate()
                .filter(|(_, reg)| *reg != &0)
            {
                writeln!(f, "x{}: {:#010x}", i, reg)?;
            }
            writeln!(f, "PC: {:#010x}", self.reg_pc_64)?;
        }
        writeln!(f, "Program Break: {:#010x}", self.program_brk)?;
        writeln!(f, "Halted: {}", self.halted)?;
        writeln!(
            f,
            "Program Memory Offset: {:#010x}",
            self.program_memory_offset
        )
    }
}

pub const INITIAL_STACK_POINTER_32: u32 = 0xbfffff00;
pub const INITIAL_STACK_POINTER_64: u64 = 0x00007FFFFFFFFFFF;
pub const KERNEL_ADDR: u64 = 0x80000000;
pub const KERNEL_SIZE: u64 = 128 * 1024 * 1024;

impl Default for Cpu {
    fn default() -> Self {
        Cpu {
            reg_x32: [0x0; 32],
            reg_x64: [0x0; 32],
            reg_f: [0.0; 32],
            reg_pc_64: 0x0,
            current_instruction_pc_64: 0x0,
            memory: Box::new(RawVecMemory::new()),
            program_cache: ProgramCache::empty(),
            program_memory_offset: 0x0,
            halted: false,
            program_brk: 0,
            kernel: Box::<PassthroughKernel>::default(),
            csr_table: CSRTable::new(CpuMode::RV32),
            arch_mode: CpuMode::RV32,
            privilege_mode: PrivilegeMode::Machine,
            pc_history: CircularBuffer::new(500),
            block_device: None,
            execution_mode: ExecutionMode::UserSpace,
            peripherals: None,
        }
    }
}

impl Cpu {
    pub fn new<M, K>(
        memory: M,
        kernel: K,
        mode: CpuMode,
        block_device: Option<BlockDevice>,
        execution_mode: ExecutionMode,
    ) -> Cpu
    where
        M: Memory + 'static,
        K: Kernel + 'static,
    {
        Cpu {
            reg_x32: [0x0; 32],
            reg_x64: [0x0; 32],
            reg_f: [0.0; 32],
            reg_pc_64: 0x0,
            current_instruction_pc_64: 0x0,
            memory: Box::new(memory),
            program_cache: ProgramCache::empty(),
            program_memory_offset: 0x0,
            halted: false,
            program_brk: 0,
            kernel: Box::new(kernel),
            csr_table: CSRTable::new(mode.clone()),
            arch_mode: mode,
            privilege_mode: PrivilegeMode::Machine,
            pc_history: CircularBuffer::new(500),
            block_device,
            execution_mode,
            peripherals: Some(Peripherals {
                uart: ContinuousMemory::new(UART_ADDR, 0x100),
                virtio: ContinuousMemory::new(VIRTIO_0_ADDR, 0x100),
                plic: ContinuousMemory::new(PLIC_ADDR, 0x201004 + 0x8),
            }),
        }
    }

    pub fn new_userspace(mode: CpuMode) -> Cpu {
        match mode {
            CpuMode::RV64 => Cpu::new(
                UserMemory::new(
                    INITIAL_STACK_POINTER_64 as u64 - STACK_SIZE,
                    0,
                    STACK_SIZE,
                    HEAP_SIZE,
                ),
                PassthroughKernel::default(),
                mode,
                None,
                ExecutionMode::UserSpace,
            ),
            CpuMode::RV32 => Cpu::new(
                UserMemory::new(
                    INITIAL_STACK_POINTER_32 as u64 - STACK_SIZE,
                    0,
                    STACK_SIZE,
                    HEAP_SIZE,
                ),
                PassthroughKernel::default(),
                mode,
                None,
                ExecutionMode::UserSpace,
            ),
        }
    }

    pub fn new_bare(block_device: Option<BlockDevice>) -> Cpu {
        Cpu::new(
            ContinuousMemory::default(),
            PassthroughKernel::default(),
            CpuMode::RV64,
            block_device,
            ExecutionMode::Bare,
        )
    }

    fn run_cycle_bare(&mut self) -> Result<()> {
        // Check if CPU is halted
        if self.halted {
            bail!("CPU is halted");
        }

        // Fetch
        #[cfg(feature = "maxperf")]
        let pc_translated = unsafe {
            self.translate_address_if_needed(self.reg_pc_64)
                .unwrap_unchecked()
        };
        #[cfg(not(feature = "maxperf"))]
        let pc_translated = self.translate_address_if_needed(self.reg_pc_64)?;

        let instruction = decode_program_line_unchecked(
            &Word(self.memory.read_mem_u32(pc_translated)?),
            self.arch_mode,
        );

        // Increase PC
        self.current_instruction_pc_64 = self.reg_pc_64;
        self.reg_pc_64 += 4;

        #[cfg(not(feature = "maxperf"))]
        self.pc_history.push((
            self.current_instruction_pc_64,
            Some(instruction.instruction),
            self.csr_table.read64(CSRAddress::Satp.as_u12()),
        ));

        // Execute
        self.execute_program_line(&instruction)?;

        plic_check_pending(self);
        check_pending_interrupts(self, PrivilegeMode::Machine);
        check_pending_interrupts(self, PrivilegeMode::Supervisor);

        Ok(())
    }

    fn run_cycle_userspace(&mut self) -> Result<()> {
        // Check if CPU is halted
        if self.halted {
            bail!("CPU is halted");
        }

        // Fetch
        let instruction = self.program_cache.get_line_unchecked(self.reg_pc_64);

        // Increase PC
        self.current_instruction_pc_64 = self.reg_pc_64;
        self.reg_pc_64 += 4;

        #[cfg(not(feature = "maxperf"))]
        self.pc_history.push((
            self.current_instruction_pc_64,
            Some(instruction.instruction),
            self.csr_table.read64(CSRAddress::Satp.as_u12()),
        ));

        // Execute
        self.execute_program_line(&instruction)?;

        Ok(())
    }

    pub fn load_program_from_elf(&mut self, elf: ElfFile) -> Result<()> {
        let program_file = load_program_to_memory(elf, self.memory.as_mut())?;

        self.reg_pc_64 = program_file.entry_point;

        let cache = ProgramCache::new(
            program_file.program_memory_offset,
            program_file.program_memory_offset + program_file.program_size,
            self.memory.as_mut(),
            self.arch_mode,
        );

        if self.execution_mode == ExecutionMode::UserSpace {
            self.program_cache = cache.unwrap();
        }
        if self.arch_mode == CpuMode::RV64 {
            self.write_x_u64(
                ABIRegister::SP.to_x_reg_id() as u8,
                INITIAL_STACK_POINTER_64,
            )
        } else {
            self.write_x_u32(
                ABIRegister::SP.to_x_reg_id() as u8,
                INITIAL_STACK_POINTER_32,
            )
        }
        self.program_brk = program_file.end_of_data_addr;

        Ok(())
    }

    pub fn load_kernel_image(&mut self, image: &mut File, addr: u64) -> Result<()> {
        self.reg_pc_64 = addr;

        load_kernel_to_memory(image, self.memory.as_mut(), addr);

        self.program_cache = ProgramCache::new(
            addr,
            addr + image.metadata().unwrap().len(),
            self.memory.as_mut(),
            self.arch_mode,
        )
        .unwrap();

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

        self.reg_pc_64 = entry_point;

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

    pub fn run_cycles(&mut self, count: u64) -> Result<()> {
        match self.execution_mode {
            ExecutionMode::Bare => {
                for _ in 0..count {
                    let res = self.run_cycle_bare();
                    if res.is_err() {
                        return res;
                    }
                }
            }
            ExecutionMode::UserSpace => {
                for _ in 0..count {
                    let res = self.run_cycle_userspace();
                    if res.is_err() {
                        return res;
                    }
                }
            }
        }

        Ok(())
    }

    #[inline(always)]
    pub fn execute_program_line(&mut self, program_line: &ProgramLine) -> Result<()> {
        let word = program_line.word;
        (program_line.instruction.operation)(self, &word)
    }

    pub fn execute_word(&mut self, word: Word) -> Result<()> {
        let program_line = decode_program_line(word, self.arch_mode)?;
        (program_line.instruction.operation)(self, &program_line.word)
    }

    pub fn set_halted(&mut self) {
        self.halted = true;
    }

    pub fn translate_address_if_needed(&mut self, addr: u64) -> Result<u64> {
        let satp = self.csr_table.read64(CSRAddress::Satp.as_u12());
        if satp != 0 {
            walk_page_table_sv39(addr, satp, self)
        } else {
            Ok(addr)
        }
    }

    pub fn read_mem_u64(&mut self, addr: u64) -> Result<u64> {
        match self.execution_mode {
            ExecutionMode::Bare => bare_read_mem_u64(self, addr),
            ExecutionMode::UserSpace => user_space_read_mem_u64(self, addr),
        }
    }

    pub fn read_mem_u32(&mut self, addr: u64) -> Result<u32> {
        match self.execution_mode {
            ExecutionMode::Bare => bare_read_mem_u32(self, addr),
            ExecutionMode::UserSpace => user_space_read_mem_u32(self, addr),
        }
    }

    pub fn read_mem_u16(&mut self, addr: u64) -> Result<u16> {
        match self.execution_mode {
            ExecutionMode::Bare => bare_read_mem_u16(self, addr),
            ExecutionMode::UserSpace => user_space_read_mem_u16(self, addr),
        }
    }

    pub fn read_mem_u8(&mut self, addr: u64) -> Result<u8> {
        match self.execution_mode {
            ExecutionMode::Bare => bare_read_mem_u8(self, addr),
            ExecutionMode::UserSpace => user_space_read_mem_u8(self, addr),
        }
    }

    pub fn write_mem_u8(&mut self, addr: u64, value: u8) -> Result<()> {
        match self.execution_mode {
            ExecutionMode::Bare => bare_write_mem_u8(self, addr, value),
            ExecutionMode::UserSpace => user_space_write_mem_u8(self, addr, value),
        }
    }

    pub fn write_mem_u16(&mut self, addr: u64, value: u16) -> Result<()> {
        match self.execution_mode {
            ExecutionMode::Bare => bare_write_mem_u16(self, addr, value),
            ExecutionMode::UserSpace => user_space_write_mem_u16(self, addr, value),
        }
    }

    pub fn write_mem_u32(&mut self, addr: u64, value: u32) -> Result<()> {
        match self.execution_mode {
            ExecutionMode::Bare => bare_write_mem_u32(self, addr, value),
            ExecutionMode::UserSpace => user_space_write_mem_u32(self, addr, value),
        }
    }

    pub fn write_mem_u64(&mut self, addr: u64, value: u64) -> Result<()> {
        match self.execution_mode {
            ExecutionMode::Bare => bare_write_mem_u64(self, addr, value),
            ExecutionMode::UserSpace => user_space_write_mem_u64(self, addr, value),
        }
    }

    pub fn read_buf(&mut self, addr: u64, buf: &mut [u8]) -> Result<()> {
        let addr = self.translate_address_if_needed(addr)?;
        self.memory.read_buf(addr, buf)
    }

    pub fn write_buf(&mut self, addr: u64, buf: &[u8]) -> Result<()> {
        let addr = self.translate_address_if_needed(addr)?;
        self.memory.write_buf(addr, buf)
    }

    #[inline(always)]
    pub fn read_x_u32(&self, id: u8) -> u32 {
        unsafe { *self.reg_x32.get_unchecked(id as usize) }
    }

    #[inline(always)]
    pub fn read_x_u64(&self, id: u8) -> u64 {
        unsafe { *self.reg_x64.get_unchecked(id as usize) }
    }

    #[inline(always)]
    pub fn read_x_i32(&self, id: u8) -> i32 {
        u32_to_i32(self.read_x_u32(id))
    }

    #[inline(always)]
    pub fn read_x_i64(&self, id: u8) -> i64 {
        u64_to_i64(self.read_x_u64(id))
    }

    #[inline(always)]
    pub fn write_x_i32(&mut self, id: u8, value: i32) {
        if id == 0 {
            return; // x0 is hardwired to 0
        }

        let reg_value = unsafe { self.reg_x32.get_unchecked_mut(id as usize) }; // SAFETY: For properly compiled code 0 <= id < 32
        *reg_value = i32_to_u32(value);
    }

    #[inline(always)]
    pub fn write_x_i64(&mut self, id: u8, value: i64) {
        if id == 0 {
            return; // x0 is hardwired to 0
        }

        let reg_value = unsafe { self.reg_x64.get_unchecked_mut(id as usize) }; // SAFETY: For properly compiled code 0 <= id < 32
        *reg_value = i64_to_u64(value);
    }

    #[inline(always)]
    pub fn write_x_u32(&mut self, id: u8, value: u32) {
        if id == 0 {
            return; // x0 is hardwired to 0
        }

        let reg_value = unsafe { self.reg_x32.get_unchecked_mut(id as usize) }; // SAFETY: For properly compiled code 0 <= id < 32
        *reg_value = value;
    }

    #[inline(always)]
    pub fn write_x_u64(&mut self, id: u8, value: u64) {
        if id == 0 {
            return; // x0 is hardwired to 0
        }

        let reg_value = unsafe { self.reg_x64.get_unchecked_mut(id as usize) }; // SAFETY: For properly compiled code 0 <= id < 32
        *reg_value = value;
    }

    pub fn write_f32(&mut self, id: u8, value: f32) {
        let reg_value = unsafe { self.reg_f.get_unchecked_mut(id as usize) }; // SAFETY: For properly compiled code 0 <= id < 32
        *reg_value = f32_to_f64(value);
    }

    pub fn write_f64(&mut self, id: u8, value: f64) {
        let reg_value = unsafe { self.reg_f.get_unchecked_mut(id as usize) }; // SAFETY: For properly compiled code 0 <= id < 32
        *reg_value = value;
    }

    pub fn read_f32(&self, id: u8) -> f32 {
        let reg_value = unsafe { self.reg_f.get_unchecked(id as usize) }; // SAFETY: For properly compiled code 0 <= id < 32
        f64_to_f32(*reg_value)
    }

    pub fn read_f64(&self, id: u8) -> f64 {
        let reg_value = unsafe { self.reg_f.get_unchecked(id as usize) }; // SAFETY: For properly compiled code 0 <= id < 32
        *reg_value
    }

    pub fn read_pc_u32(&self) -> u32 {
        self.reg_pc_64 as u32
    }

    pub fn read_pc_u64(&self) -> u64 {
        self.reg_pc_64
    }

    pub fn write_pc_u32(&mut self, val: u32) {
        self.reg_pc_64 = val as u64;
    }

    pub fn write_pc_u64(&mut self, val: u64) {
        self.reg_pc_64 = val;
    }

    #[allow(unused)]
    fn print_breakpoint(&mut self, pc: u64, val: u64, name: &str) -> bool {
        if val == pc {
            println!(
                "jump to {:#x} from {:#x} into {}",
                pc, self.current_instruction_pc_64, name
            );
            println!();
            return true;
        }
        false
    }

    pub fn read_current_instruction_addr_u32(&self) -> u32 {
        self.current_instruction_pc_64 as u32
    }

    pub fn read_current_instruction_addr_u64(&self) -> u64 {
        self.current_instruction_pc_64
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
