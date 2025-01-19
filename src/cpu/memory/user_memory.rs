#[allow(unused_imports)]
use anyhow::{bail, Result};

use crate::cpu::cpu_core::INITIAL_STACK_POINTER_32;

use super::{memory_core::Memory, raw_memory::ContinuousMemory};

#[derive(Debug)]
pub struct UserMemory {
    stack: ContinuousMemory,
    heap: ContinuousMemory,
}

pub const STACK_SIZE: u64 = 0x10000;
pub const HEAP_SIZE: u64 = 0x1000000;

impl UserMemory {
    pub fn new(stack_addr: u64, heap_addr: u64, stack_size: u64, heap_size: u64) -> Self {
        Self {
            stack: ContinuousMemory::new(stack_addr, stack_size),
            heap: ContinuousMemory::new(heap_addr, heap_size),
        }
    }
}

impl Default for UserMemory {
    fn default() -> Self {
        Self::new(
            INITIAL_STACK_POINTER_32 as u64 - STACK_SIZE,
            0x0,
            STACK_SIZE,
            HEAP_SIZE,
        )
    }
}

const CUTOFF_ADDR: u64 = 0x10000000;
impl Memory for UserMemory {
    fn read_mem_u8(&mut self, addr: u64) -> Result<u8> {
        if addr >= CUTOFF_ADDR {
            self.stack.read_mem_u8(addr)
        } else {
            self.heap.read_mem_u8(addr)
        }
    }

    fn read_mem_u16(&mut self, addr: u64) -> Result<u16> {
        if addr >= CUTOFF_ADDR {
            self.stack.read_mem_u16(addr)
        } else {
            self.heap.read_mem_u16(addr)
        }
    }

    fn read_mem_u32(&mut self, addr: u64) -> Result<u32> {
        if addr >= CUTOFF_ADDR {
            self.stack.read_mem_u32(addr)
        } else {
            self.heap.read_mem_u32(addr)
        }
    }

    fn read_mem_u64(&mut self, addr: u64) -> Result<u64> {
        if addr >= CUTOFF_ADDR {
            self.stack.read_mem_u64(addr)
        } else {
            self.heap.read_mem_u64(addr)
        }
    }

    fn write_mem_u8(&mut self, addr: u64, value: u8) -> Result<()> {
        if addr >= CUTOFF_ADDR {
            self.stack.write_mem_u8(addr, value)
        } else {
            self.heap.write_mem_u8(addr, value)
        }
    }

    fn write_mem_u16(&mut self, addr: u64, value: u16) -> Result<()> {
        if addr >= CUTOFF_ADDR {
            self.stack.write_mem_u16(addr, value)
        } else {
            self.heap.write_mem_u16(addr, value)
        }
    }

    fn write_mem_u32(&mut self, addr: u64, value: u32) -> Result<()> {
        if addr >= CUTOFF_ADDR {
            self.stack.write_mem_u32(addr, value)
        } else {
            self.heap.write_mem_u32(addr, value)
        }
    }

    fn write_mem_u64(&mut self, addr: u64, value: u64) -> Result<()> {
        if addr >= CUTOFF_ADDR {
            self.stack.write_mem_u64(addr, value)
        } else {
            self.heap.write_mem_u64(addr, value)
        }
    }

    fn read_buf(&mut self, addr: u64, buf: &mut [u8]) -> Result<()> {
        if addr >= CUTOFF_ADDR {
            self.stack.read_buf(addr, buf)
        } else {
            self.heap.read_buf(addr, buf)
        }
    }

    fn write_buf(&mut self, addr: u64, buf: &[u8]) -> Result<()> {
        if addr >= CUTOFF_ADDR {
            self.stack.write_buf(addr, buf)
        } else {
            self.heap.write_buf(addr, buf)
        }
    }
}
