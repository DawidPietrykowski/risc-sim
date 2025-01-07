use std::{fmt::Display, fs::File};

use crate::{
    elf::elf_loader::{load_kernel_to_memory, load_program_to_memory, ElfFile},
    isa::{
        csr::csr_types::{CSRAddress, CSRTable, MisaCSR},
        traps::check_pending_interrupts,
    },
    system::{
        kernel::Kernel,
        passthrough_kernel::PassthroughKernel,
        plic::{
            plic_check_pending, plic_handle_claim_read, plic_handle_claim_write,
            plic_handle_pending_write, PLIC_CLAIM, PLIC_PENDING,
        },
        uart::{uart_handle_read, uart_handle_write, UART_ADDR},
        virtio::{process_queue, BlockDevice, VIRTIO_0_ADDR, VIRTIO_MMIO_QUEUE_NOTIFY},
    },
    types::{ABIRegister, Instruction},
    utils::binary_utils::*,
};

use super::memory::{
    memory_core::Memory, mmu::walk_page_table_sv39, program_cache::ProgramCache,
    raw_vec_memory::RawVecMemory,
};
use crate::types::{decode_program_line, ProgramLine, Word};
use anyhow::{bail, Context, Ok, Result};

#[derive(PartialEq, Clone, Copy)]
pub enum CpuMode {
    RV32,
    RV64,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum PrivilegeMode {
    User = 0,
    Supervisor = 1,
    Machine = 3,
}

pub struct CircularBuffer<T> {
    buffer: Vec<T>,
    head: usize,
    tail: usize,
    size: usize,
}

impl<T> CircularBuffer<T>
where
    T: Default,
    T: Clone,
{
    fn new(size: usize) -> Self {
        CircularBuffer {
            buffer: vec![Default::default(); size],
            head: 0,
            tail: 0,
            size,
        }
    }

    fn push(&mut self, item: T) {
        self.buffer[self.head] = item;
        self.head = (self.head + 1) % self.size;
        if self.head == self.tail {
            self.tail = (self.tail + 1) % self.size;
        }
    }

    fn pop(&mut self) -> Option<T> {
        if self.tail != self.head {
            let item = self.buffer[self.tail].clone();
            self.tail = (self.tail + 1) % self.size;
            Some(item)
        } else {
            None
        }
    }
}

struct ExecutionVTable {
    run_cycles: fn(cpu: &mut Cpu) -> Result<()>,
    update_pc: fn(&mut Cpu),
    get_current_pc: fn(&Cpu) -> u64,
    get_current_pc_translated: fn(&mut Cpu) -> u64,
    fetch_instruction: fn(&mut Cpu) -> ProgramLine,
}

pub struct Cpu {
    reg_x32: [u32; 32],
    reg_x64: [u64; 32],
    reg_f: [f64; 32],
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
    pub pc_history: CircularBuffer<(u64, Option<Instruction>, u64)>,
    pub block_device: Option<BlockDevice>,
    vtable: ExecutionVTable,
    pub execution_mode: ExecutionMode,
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
            writeln!(f, "PC: {:#010x}", self.reg_pc)?;
        }
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
            reg_f: [0.0; 32],
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
            pc_history: CircularBuffer::new(500),
            block_device: None,
            vtable: ExecutionVTable::new(CpuMode::RV32, ExecutionMode::UserSpace, false),
            execution_mode: ExecutionMode::UserSpace,
        };
        cpu.setup_csrs();
        cpu
    }
}

impl ExecutionVTable {
    fn new(mode: CpuMode, execution_mode: ExecutionMode, force_cache: bool) -> Self {
        let run_cycles = match execution_mode {
            ExecutionMode::Bare => |cpu: &mut Cpu| {
                // Check if CPU is halted
                if cpu.halted {
                    bail!("CPU is halted");
                }

                // Fetch
                #[cfg(feature = "maxperf")]
                let instruction = (cpu.vtable.fetch_instruction)(cpu);
                #[cfg(not(feature = "maxperf"))]
                let instruction = cpu.fetch_instruction()?;

                #[cfg(not(feature = "maxperf"))]
                if cpu.debug_enabled {
                    println!("\nPC({:#x}) {}", cpu.reg_pc, instruction);
                }

                // Increase PC
                (cpu.vtable.update_pc)(cpu);

                #[cfg(not(feature = "maxperf"))]
                cpu.pc_history.push((
                    cpu.current_instruction_pc_64,
                    Some(instruction.instruction),
                    cpu.csr_table.read64(CSRAddress::Satp.as_u12()),
                ));

                // Execute
                #[cfg(feature = "maxperf")]
                let _ = cpu.execute_program_line(&instruction);
                #[cfg(not(feature = "maxperf"))]
                cpu.execute_program_line(&instruction)?;

                plic_check_pending(cpu);
                check_pending_interrupts(cpu, PrivilegeMode::Machine);
                check_pending_interrupts(cpu, PrivilegeMode::Supervisor);

                Ok(())
            },
            ExecutionMode::UserSpace => |cpu: &mut Cpu| {
                // Check if CPU is halted
                #[cfg(not(feature = "maxperf"))]
                if cpu.halted {
                    bail!("CPU is halted");
                }

                // Fetch
                #[cfg(feature = "maxperf")]
                let instruction = (cpu.vtable.fetch_instruction)(cpu);
                #[cfg(not(feature = "maxperf"))]
                let instruction = cpu.fetch_instruction()?;

                #[cfg(not(feature = "maxperf"))]
                if cpu.debug_enabled {
                    println!("\nPC({:#x}) {}", cpu.reg_pc, instruction);
                }

                // Increase PC
                (cpu.vtable.update_pc)(cpu);

                #[cfg(not(feature = "maxperf"))]
                cpu.pc_history.push((
                    cpu.current_instruction_pc_64,
                    Some(instruction.instruction),
                    cpu.csr_table.read64(CSRAddress::Satp.as_u12()),
                ));

                // Execute
                #[cfg(feature = "maxperf")]
                let _ = cpu.execute_program_line(&instruction);
                #[cfg(not(feature = "maxperf"))]
                cpu.execute_program_line(&instruction)?;

                Ok(())
            },
        };
        let fetch_instruction = match force_cache {
            true => |cpu: &mut Cpu| {
                let mut pc = (cpu.vtable.get_current_pc_translated)(cpu);
                cpu.program_cache.get_line_unchecked(pc)
            },
            false => |cpu: &mut Cpu| unsafe {
                let mut pc = (cpu.vtable.get_current_pc_translated)(cpu);
                decode_program_line(
                    Word(cpu.memory.read_mem_u32(pc).unwrap_unchecked()),
                    cpu.arch_mode,
                )
                .unwrap_unchecked()
            },
        };
        let get_current_pc_translated = match execution_mode {
            ExecutionMode::Bare => {
                if cfg!(feature = "maxperf") {
                    |cpu: &mut Cpu| unsafe {
                        cpu.translate_address_if_needed(cpu.reg_pc_64)
                            .unwrap_unchecked()
                    }
                } else {
                    |cpu: &mut Cpu| cpu.translate_address_if_needed(cpu.reg_pc_64).unwrap()
                }
            }
            ExecutionMode::UserSpace => |cpu: &mut Cpu| cpu.reg_pc_64,
        };
        match mode {
            CpuMode::RV64 => Self {
                update_pc: |cpu: &mut Cpu| {
                    cpu.current_instruction_pc_64 = cpu.reg_pc_64;
                    cpu.reg_pc_64 += 4;
                },
                get_current_pc: |cpu: &Cpu| cpu.reg_pc_64,
                get_current_pc_translated,
                run_cycles,
                fetch_instruction,
            },
            CpuMode::RV32 => Self {
                update_pc: |cpu: &mut Cpu| {
                    cpu.current_instruction_pc = cpu.reg_pc;
                    cpu.reg_pc += 4;
                },
                get_current_pc: |cpu: &Cpu| cpu.reg_pc as u64,
                get_current_pc_translated,
                run_cycles,
                fetch_instruction,
            },
        }
    }
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum ExecutionMode {
    UserSpace,
    Bare,
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
        let mut cpu = Cpu {
            reg_x32: [0x0; 32],
            reg_x64: [0x0; 32],
            reg_f: [0.0; 32],
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
            pc_history: CircularBuffer::new(500),
            block_device,
            vtable: ExecutionVTable::new(
                mode,
                execution_mode.clone(),
                execution_mode == ExecutionMode::UserSpace,
            ),
            execution_mode,
        };
        cpu.setup_csrs();
        cpu
    }

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
        .unwrap_or(ProgramCache::empty());
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
        return (self.vtable.run_cycles)(self);

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

        self.pc_history.push((
            self.current_instruction_pc_64,
            Some(instruction.instruction),
            self.csr_table.read64(CSRAddress::Satp.as_u12()),
        ));

        // Execute
        self.execute_program_line(&instruction)?;

        if !self.simulate_kernel {
            plic_check_pending(self);
            check_pending_interrupts(self, PrivilegeMode::Machine);
            check_pending_interrupts(self, PrivilegeMode::Supervisor);
        }

        Ok(())
    }

    #[cfg(feature = "maxperf")]
    pub fn run_cycle_uncheked(&mut self) -> Result<()> {
        return (self.vtable.run_cycles)(self);
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

        self.pc_history.push((
            self.current_instruction_pc_64,
            Some(instruction.instruction),
            self.csr_table.read64(CSRAddress::Satp.as_u12()),
        ));

        // Execute
        let _ = self.execute_program_line(&instruction);

        if !self.simulate_kernel {
            plic_check_pending(self);
            check_pending_interrupts(self, PrivilegeMode::Machine);
            check_pending_interrupts(self, PrivilegeMode::Supervisor);
        }

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
        let mut pc = self.read_pc();
        pc = self.translate_address_if_needed(pc)?;
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
    fn fetch_instruction_unchecked(&mut self) -> ProgramLine {
        let mut pc = self.read_pc();
        pc = self.translate_address_if_needed(pc).unwrap();
        self.program_cache.get_line_unchecked(pc)
        //decode_program_line(Word(self.memory.read_mem_u32(pc).unwrap()), self.arch_mode).unwrap()
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
        let addr = self.translate_address_if_needed(addr)?;
        self.memory.read_mem_u64(addr)
    }

    pub fn read_mem_u32(&mut self, addr: u64) -> Result<u32> {
        let addr = self.translate_address_if_needed(addr)?;
        if addr == PLIC_CLAIM {
            return Ok(plic_handle_claim_read(self));
        }
        self.memory.read_mem_u32(addr)
    }

    pub fn read_mem_u16(&mut self, addr: u64) -> Result<u16> {
        let addr = self.translate_address_if_needed(addr)?;
        self.memory.read_mem_u16(addr)
    }

    pub fn read_mem_u8(&mut self, addr: u64) -> Result<u8> {
        let addr = self.translate_address_if_needed(addr)?;
        if addr == UART_ADDR {
            return Ok(uart_handle_read(self) as u8);
        }
        self.memory.read_mem_u8(addr)
    }

    pub fn write_mem_u8(&mut self, addr: u64, value: u8) -> Result<()> {
        let addr = self.translate_address_if_needed(addr)?;
        // TODO: Add better mechanism for hooks
        if addr == UART_ADDR {
            uart_handle_write(self, value);
            return Ok(());
        }
        self.memory.write_mem_u8(addr, value)
    }

    pub fn write_mem_u16(&mut self, addr: u64, value: u16) -> Result<()> {
        let addr = self.translate_address_if_needed(addr)?;
        self.memory.write_mem_u16(addr, value)
    }

    pub fn write_mem_u32(&mut self, addr: u64, value: u32) -> Result<()> {
        let addr = self.translate_address_if_needed(addr)?;
        if addr == VIRTIO_0_ADDR + VIRTIO_MMIO_QUEUE_NOTIFY as u64 {
            process_queue(self);
        }
        if addr == PLIC_PENDING {
            plic_handle_pending_write(self, value);
        }
        if addr == PLIC_CLAIM {
            plic_handle_claim_write(self, value);
            return Ok(());
        }
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
            .context("Register does not exist")?;
        #[cfg(feature = "maxperf")]
        let reg_value = unsafe { self.reg_x32.get_unchecked_mut(id as usize) }; // SAFETY: For properly compiled code 0 <= id < 32

        *reg_value = value;
        Ok(())
    }

    pub fn print_pc_history(&mut self) {
        println!("pc history:");
        let mut last_pc = 0u64;
        while let Some((pc, ins, satp)) = self.pc_history.pop() {
            if pc != last_pc + 0x4 {
                println!("jmp");
            }
            println!("{:x} {} {:x}", pc, ins.unwrap().name, satp);
            last_pc = pc;
        }
        println!();
    }

    pub fn write_x_u64(&mut self, id: u8, value: u64) -> Result<()> {
        if id == 0 {
            return Ok(()); // x0 is hardwired to 0
        }

        #[cfg(not(feature = "maxperf"))]
        let reg_value = self
            .reg_x64
            .get_mut(id as usize)
            .context("Register does not exist")?;
        #[cfg(feature = "maxperf")]
        let reg_value = unsafe { self.reg_x64.get_unchecked_mut(id as usize) }; // SAFETY: For properly compiled code 0 <= id < 32

        *reg_value = value;
        Ok(())
    }

    pub fn write_f32(&mut self, id: u8, value: f32) -> Result<()> {
        #[cfg(not(feature = "maxperf"))]
        let reg_value = self
            .reg_f
            .get_mut(id as usize)
            .context("Register does not exist")?;
        #[cfg(feature = "maxperf")]
        let reg_value = unsafe { self.reg_f.get_unchecked_mut(id as usize) }; // SAFETY: For properly compiled code 0 <= id < 32
        *reg_value = f32_to_f64(value);
        Ok(())
    }

    pub fn write_f64(&mut self, id: u8, value: f64) -> Result<()> {
        #[cfg(not(feature = "maxperf"))]
        let reg_value = self
            .reg_f
            .get_mut(id as usize)
            .context("Register does not exist")?;
        #[cfg(feature = "maxperf")]
        let reg_value = unsafe { self.reg_f.get_unchecked_mut(id as usize) }; // SAFETY: For properly compiled code 0 <= id < 32
        *reg_value = value;
        Ok(())
    }

    pub fn read_f32(&self, id: u8) -> Result<f32> {
        #[cfg(not(feature = "maxperf"))]
        let reg_value = self
            .reg_f
            .get(id as usize)
            .context("Register does not exist")?;
        #[cfg(feature = "maxperf")]
        let reg_value = unsafe { self.reg_f.get_unchecked(id as usize) }; // SAFETY: For properly compiled code 0 <= id < 32
        Ok(f64_to_f32(*reg_value))
    }

    pub fn read_f64(&self, id: u8) -> Result<f64> {
        #[cfg(not(feature = "maxperf"))]
        let reg_value = self
            .reg_f
            .get(id as usize)
            .context("Register does not exist")?;
        #[cfg(feature = "maxperf")]
        let reg_value = unsafe { self.reg_f.get_unchecked(id as usize) }; // SAFETY: For properly compiled code 0 <= id < 32
        Ok(*reg_value)
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
        // TODO: Remove
        // self.print_breakpoint(0x80000a54, val, "printfinit");
        // self.print_breakpoint(0x80000540, val, "consoleinit");
        // self.print_breakpoint(0x80002aa4, val, "scheduler");
        // self.print_breakpoint(0x80000db4, val, "kfree");
        // self.print_breakpoint(0x80000ebc, val, "kinit");
        // self.print_breakpoint(0x80000e44, val, "freerange");
        //self.print_breakpoint(0x800034d0, val, "usertrapret");
        //self.print_breakpoint(0x800090b0, val, "userret");
        //self.print_breakpoint(0x80009000, val, "trampoline");
        //self.print_breakpoint(0x8000146c, val, "main");
        //self.print_breakpoint(0x80003824, val, "kerneltrap");
        //self.print_breakpoint(0x0000000080001404, val, "scheduler");
        //self.print_breakpoint(0x8000167c, val, "mappages");
        //if self.print_breakpoint(0x00000000800033fc, val, "swtch") {
        //    let ra = self
        //        .read_x_u64(ABIRegister::RA.to_x_reg_id() as u8)
        //        .unwrap();
        //    println!("ra: {:#x}", ra);
        //    let sp = self
        //        .read_x_u64(ABIRegister::SP.to_x_reg_id() as u8)
        //        .unwrap();
        //    println!("sp: {:#x}", sp);
        //    println!();
        //}
        //if self.print_breakpoint(0x80002500, val, "back from swtch") {
        //    let ra = self
        //        .read_x_u64(ABIRegister::RA.to_x_reg_id() as u8)
        //        .unwrap();
        //    println!("ra: {:#x}", ra);
        //    let sp = self
        //        .read_x_u64(ABIRegister::SP.to_x_reg_id() as u8)
        //        .unwrap();
        //    println!("sp: {:#x}", sp);
        //    println!();
        //}
        //self.print_breakpoint(0x0000000080002864, val, "userinit");
        // if self.current_instruction_pc_64 != 0x80000e1c {
        // self.print_breakpoint(0x8000112c, val, "release");
        // }
        // if self.current_instruction_pc_64 != 0x8000115c {
        // self.print_breakpoint(0x800010b8, val, "pop_off");
        // }
        // self.print_breakpoint(0x8000466c, val, "ialoc");
        // self.print_breakpoint(0x80000664, val, "printf");
        // self.print_breakpoint(0x80000a08, val, "panic");

        //if self.print_breakpoint(0x0, val, "zero") {
        //    let ra = self
        //        .read_x_u64(ABIRegister::RA.to_x_reg_id() as u8)
        //        .unwrap();
        //    println!("ra: {:#x}", ra);
        //    let sp = self
        //        .read_x_u64(ABIRegister::SP.to_x_reg_id() as u8)
        //        .unwrap();
        //    println!("sp: {:#x}", sp);
        //    println!();
        //    // panic!()
        //}
        self.reg_pc_64 = val;
    }

    // TODO: add better mechanism
    //fn print_breakpoint(&mut self, pc: u64, val: u64, name: &str) -> bool {
    //    if val == pc {
    //        println!(
    //            "jump to {:#x} from {:#x} into {}",
    //            pc, self.current_instruction_pc_64, name
    //        );
    //        println!();
    //        return true;
    //    }
    //    false
    //}

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
