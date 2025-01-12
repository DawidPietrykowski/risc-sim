use anyhow::{Ok, Result};
use std::{
    fmt::{Debug, Formatter},
    slice,
};

use super::memory_core::Memory;
use super::page_storage::PAGE_SIZE;

#[allow(unused)]
pub trait RawPageStorage {
    fn new() -> Self;
    fn get_page_id(&self, addr: u64) -> u64;
    fn get_page(&self, page_id: u64) -> Option<&Page>;
    fn get_page_mut(&mut self, page_id: u64) -> Option<&mut Page>;
    fn get_page_or_create(&mut self, page_id: u64) -> &mut Page;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: RawPageStorage> Debug for RawPageMemory<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Memory {{ pages: {} }}", self.storage.len())
    }
}

#[derive(Clone)]
pub struct Page {
    pub data: Box<[u8; PAGE_SIZE as usize]>,
    pub position: u64,
}

impl Page {
    pub fn new(position: u64) -> Page {
        Page {
            position,
            data: unsafe { Box::new_zeroed().assume_init() },
        }
    }
}

impl<T: RawPageStorage> Default for RawPageMemory<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: RawPageStorage> RawPageMemory<T> {
    pub fn new() -> Self {
        RawPageMemory { storage: T::new() }
    }

    fn write_u32_to_page(&mut self, addr: u64, value: u32) -> Result<()> {
        let page_id = self.storage.get_page_id(addr);

        let page = self.storage.get_page_or_create(page_id);
        if (addr + 3 - page.position) >= PAGE_SIZE {
            let upper_page_bytes = addr & 0b11;
            let lower_page_bytes = 4 - upper_page_bytes;

            unsafe {
                let value_bytes: [u8; 4] = value.to_le_bytes();

                {
                    let lower_local_addr = addr - page.position;
                    let ptr: *mut u8 = page.data.as_mut_ptr().add(lower_local_addr as usize);
                    let bytes = slice::from_raw_parts_mut(ptr, lower_page_bytes as usize);
                    bytes.copy_from_slice(&value_bytes[..lower_page_bytes as usize]);
                }
                {
                    let upper_page = self.storage.get_page_or_create(page_id + 1);
                    let ptr: *mut u8 = upper_page.data.as_mut_ptr();
                    let bytes = slice::from_raw_parts_mut(ptr, upper_page_bytes as usize);
                    bytes.copy_from_slice(&value_bytes[lower_page_bytes as usize..]);
                }
            }
        } else {
            unsafe {
                let ptr = page
                    .data
                    .as_ptr()
                    .add(addr as usize - page.position as usize);
                let bytes = ptr as *mut [u8; 4];
                bytes.write(value.to_le_bytes());
            }
        }
        Ok(())
    }

    fn read_u32_from_page(&mut self, addr: u64) -> Result<u32> {
        let page_id = self.storage.get_page_id(addr);
        if let Some(page) = self.storage.get_page(page_id) {
            if (addr + 3 - page.position) >= PAGE_SIZE {
                let upper_page_bytes = (addr & 0b11) as usize;
                let lower_page_bytes = 4 - upper_page_bytes;

                let mut res_bytes: [u8; 4] = [0u8; 4];

                unsafe {
                    {
                        let lower_local_addr = (addr - page.position) as usize;
                        let ptr: *const u8 = page.data.as_ptr().add(lower_local_addr);
                        let bytes = slice::from_raw_parts(ptr, lower_page_bytes);
                        res_bytes[..lower_page_bytes].copy_from_slice(bytes);
                    }
                    if let Some(upper_page) = self.storage.get_page(page_id + 1) {
                        let ptr: *const u8 = upper_page.data.as_ptr();
                        let bytes = slice::from_raw_parts(ptr, upper_page_bytes);
                        res_bytes[lower_page_bytes..].copy_from_slice(bytes);
                    }

                    Ok(u32::from_le_bytes(res_bytes))
                }
            } else {
                unsafe {
                    let ptr = page.data.as_ptr().add((addr - page.position) as usize);
                    let bytes = ptr as *const [u8; 4];
                    Ok(u32::from_le_bytes(bytes.read()))
                }
            }
        } else {
            Ok(0)
        }
    }
}

#[derive(Clone, Copy)]
pub struct RawPageMemory<T: RawPageStorage> {
    storage: T,
}

impl<T: RawPageStorage> Memory for RawPageMemory<T> {
    fn read_mem_u8(&mut self, addr: u64) -> Result<u8> {
        if let Some(page) = self.storage.get_page(self.storage.get_page_id(addr)) {
            let local_addr = (addr - page.position) as usize;
            unsafe { Ok(*page.data.get_unchecked(local_addr)) }
        } else {
            Ok(0)
        }
    }

    fn read_mem_u32(&mut self, addr: u64) -> Result<u32> {
        self.read_u32_from_page(addr)
    }

    fn read_mem_u16(&mut self, addr: u64) -> Result<u16> {
        let page_id = self.storage.get_page_id(addr);
        if let Some(page) = self.storage.get_page(page_id) {
            if (addr % PAGE_SIZE) == PAGE_SIZE - 1 {
                let mut res_bytes: [u8; 2] = [0u8; 2];

                unsafe {
                    {
                        let lower_local_addr = (PAGE_SIZE - 1) as usize;
                        *res_bytes.get_unchecked_mut(0) =
                            *page.data.get_unchecked(lower_local_addr);
                    }
                    if let Some(upper_page) = self.storage.get_page(page_id + 1) {
                        *res_bytes.get_unchecked_mut(1) = *upper_page.data.get_unchecked(0);
                    }

                    Ok(u16::from_le_bytes(res_bytes))
                }
            } else {
                unsafe {
                    let ptr = page.data.as_ptr().add((addr - page.position) as usize);
                    let bytes = ptr as *const [u8; 2];
                    Ok(u16::from_le_bytes(bytes.read()))
                }
            }
        } else {
            Ok(0)
        }
    }

    fn write_mem_u8(&mut self, addr: u64, value: u8) -> Result<()> {
        let page = self
            .storage
            .get_page_or_create(self.storage.get_page_id(addr));
        let local_addr = (addr - page.position) as usize;
        unsafe {
            *page.data.get_unchecked_mut(local_addr) = value;
        }
        Ok(())
    }

    fn write_mem_u16(&mut self, addr: u64, value: u16) -> Result<()> {
        let page_id = self.storage.get_page_id(addr);
        let page = self.storage.get_page_or_create(page_id);
        if (addr % PAGE_SIZE) == PAGE_SIZE - 1 {
            let mut data_bytes: [u8; 2] = value.to_le_bytes();

            unsafe {
                {
                    let lower_local_addr = (PAGE_SIZE - 1) as usize;
                    *page.data.get_unchecked_mut(lower_local_addr) =
                        *data_bytes.get_unchecked_mut(0);
                }
                {
                    let upper_page = self.storage.get_page_or_create(page_id + 1);
                    *upper_page.data.get_unchecked_mut(0) = *data_bytes.get_unchecked_mut(1);
                }
            }
        } else {
            unsafe {
                let ptr = page.data.as_ptr().add((addr - page.position) as usize);
                let bytes = ptr as *mut [u8; 2];
                bytes.write(value.to_le_bytes());
            }
        }

        Ok(())
    }

    fn write_mem_u32(&mut self, addr: u64, value: u32) -> Result<()> {
        self.write_u32_to_page(addr, value)
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

    fn read_mem_u64(&mut self, addr: u64) -> Result<u64> {
        let lower_u32 = self.read_u32_from_page(addr)? as u64;
        let upper_u32 = self.read_u32_from_page(addr + 4)? as u64;
        Ok(lower_u32 | (upper_u32 << 32))
        // TODO: implement more optimized u64 read
    }

    fn write_mem_u64(&mut self, addr: u64, value: u64) -> Result<()> {
        self.write_u32_to_page(addr, (value & 0xFFFF_FFFF) as u32)?;
        self.write_u32_to_page(addr + 4, (value >> 32) as u32)?;
        Ok(())
        // TODO: implement more optimized u64 write
    }
}
