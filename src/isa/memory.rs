use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};

const PAGE_SIZE: u32 = 4096;

const MEMORY_SIZE: u32 = u32::MAX;

pub struct Memory {
    pages: HashMap<u32, Page>
}

struct Page {
    data: [u8; PAGE_SIZE as usize],
    position: u32,
}

impl Page {
    pub fn new(position: u32) -> Page {
        Page {
            position: position,
            data: [0; PAGE_SIZE as usize]
        }
    }
}

impl Memory {
    pub fn new() -> Self {
        Memory { pages: HashMap::new() }
    }

    pub fn read_mem_u8(&self, addr: u32) -> Result<u8> {
        if addr == u32::MAX {
            return Err(anyhow!("Tried to access outside of memory bounds"));
        }
        if let Some(page) = self.pages.get(&(addr / PAGE_SIZE)) {
            Ok(page.data[(addr - page.position) as usize])
        } else {
            Ok(0)
        }
    }

    pub fn read_mem_u32(&self, addr: u32) -> Result<u32> {
        Ok((self.read_mem_u16(addr)? as u32) | (self.read_mem_u16(addr + 2)? as u32) << 16)
    }

    pub fn read_mem_u16(&self, addr: u32) -> Result<u16> {
        Ok((self.read_mem_u8(addr)? as u16) | (self.read_mem_u8(addr + 1)? as u16) << 8)
    }

    pub fn write_mem_u8(&mut self, addr: u32, value: u8) -> Result<()> {
        let page_id = addr / PAGE_SIZE;
        if page_id > MEMORY_SIZE / PAGE_SIZE {
            return Err(anyhow!("Tried to access outside of memory bounds"));
        }

        if !self.pages.contains_key(&page_id) {
            self.pages.insert(page_id, Page::new(page_id * PAGE_SIZE));
        }

        let page = self.pages.get_mut(&(addr / PAGE_SIZE)).context("Failed to allocate new page")?;

        page.data[(addr - page.position) as usize] = value;

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
}