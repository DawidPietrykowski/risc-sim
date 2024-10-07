use std::{fmt::Debug, fmt::Formatter};

use anyhow::{Context, Result};

use rustc_hash::{FxBuildHasher, FxHashMap};

use super::memory_core::Memory;

const PAGE_SIZE: u64 = 4096 * 16;

#[derive(Clone)]
pub struct VecU8Memory {
    pages: FxHashMap<u64, PageU8>,
}

impl Debug for VecU8Memory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Memory {{ pages: {} }}", self.pages.len())
    }
}

#[derive(Clone)]
struct PageU8 {
    data: [u8; PAGE_SIZE as usize],
    position: u64,
}

impl PageU8 {
    pub fn new(position: u64) -> PageU8 {
        PageU8 {
            position,
            data: [0; PAGE_SIZE as usize],
        }
    }
}

impl VecU8Memory {
    pub fn new() -> Self {
        VecU8Memory {
            pages: FxHashMap::with_capacity_and_hasher(1024, FxBuildHasher),
        }
    }
}

impl Default for VecU8Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory for VecU8Memory {
    fn read_mem_u8(&mut self, addr: u64) -> Result<u8> {
        if let Some(page) = self.pages.get(&(addr / PAGE_SIZE)) {
            Ok(page.data[(addr - page.position) as usize])
        } else {
            Ok(0)
        }
    }

    fn read_mem_u32(&mut self, addr: u64) -> Result<u32> {
        Ok((self.read_mem_u16(addr)? as u32) | (self.read_mem_u16(addr + 2)? as u32) << 16)
    }

    fn read_mem_u16(&mut self, addr: u64) -> Result<u16> {
        Ok((self.read_mem_u8(addr)? as u16) | (self.read_mem_u8(addr + 1)? as u16) << 8)
    }

    fn write_mem_u8(&mut self, addr: u64, value: u8) -> Result<()> {
        let page_id = addr / PAGE_SIZE;

        self.pages
            .entry(page_id)
            .or_insert_with(|| PageU8::new(page_id * PAGE_SIZE));

        let page = self
            .pages
            .get_mut(&(addr / PAGE_SIZE))
            .context("Failed to allocate new page")?;

        page.data[(addr - page.position) as usize] = value;

        Ok(())
    }

    fn write_mem_u16(&mut self, addr: u64, value: u16) -> Result<()> {
        self.write_mem_u8(addr, value as u8)?;
        self.write_mem_u8(addr + 1, (value >> 8) as u8)?;
        Ok(())
    }

    fn write_mem_u32(&mut self, addr: u64, value: u32) -> Result<()> {
        self.write_mem_u16(addr, value as u16)?;
        self.write_mem_u16(addr + 2, (value >> 16) as u16)?;
        Ok(())
    }

    fn read_buf(&mut self, addr: u64, buf: &mut [u8]) -> Result<()> {
        for (i, byte) in buf.iter_mut().enumerate() {
            *byte = self.read_mem_u8(addr + i as u64)?;
        }
        Ok(())
    }

    fn write_buf(&mut self, addr: u64, buf: &[u8]) -> Result<()> {
        for (i, byte) in buf.iter().enumerate() {
            self.write_mem_u8(addr + i as u64, *byte)?;
        }
        Ok(())
    }
}
