use anyhow::{Ok, Result};
use std::{
    fmt::{Debug, Formatter},
    slice,
};

use super::memory_core::Memory;

pub(crate) const PAGE_SIZE_LOG2: u32 = 18;
pub const PAGE_SIZE: u32 = 1 << PAGE_SIZE_LOG2;

#[allow(unused)]
pub trait RawPageStorage {
    fn new() -> Self;
    fn get_page_id(&self, addr: u32) -> u32;
    fn get_page(&self, page_id: u32) -> Option<&Page>;
    fn get_page_mut(&mut self, page_id: u32) -> Option<&mut Page>;
    fn get_page_or_create(&mut self, page_id: u32) -> &mut Page;
    fn len(&self) -> usize;
}

impl<T: RawPageStorage> Debug for RawPageMemory<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Memory {{ pages: {} }}", self.storage.len())
    }
}

pub struct Page {
    pub data: Box<[u8; PAGE_SIZE as usize]>,
    pub position: u32,
}

impl Page {
    pub fn new(position: u32) -> Page {
        Page {
            position,
            data: Box::new([0; PAGE_SIZE as usize]),
        }
    }
}

impl<T: RawPageStorage> RawPageMemory<T> {
    pub fn new() -> Self {
        RawPageMemory { storage: T::new() }
    }

    fn write_u32_to_page(&mut self, addr: u32, value: u32) -> Result<()> {
        let page_id = self.storage.get_page_id(addr);

        let page = self.storage.get_page_or_create(page_id);
        if addr as u64 + 3 - page.position as u64 >= PAGE_SIZE as u64 {
            let offset = addr & 0b11;

            unsafe {
                let value_bytes: [u8; 4] = value.to_le_bytes();

                let lower_local_addr = addr - page.position;
                let ptr: *mut u8 = page.data.as_mut_ptr().add(lower_local_addr as usize);
                let bytes = slice::from_raw_parts_mut(ptr, 4 - offset as usize);
                bytes[..4 - offset as usize].copy_from_slice(&value_bytes[offset as usize..]);

                if let Some(upper_page) = self.storage.get_page_mut(page_id + 1) {
                    let upper_local_addr = addr - upper_page.position;
                    let ptr: *mut u8 = upper_page.data.as_mut_ptr().add(upper_local_addr as usize);
                    let bytes = slice::from_raw_parts_mut(ptr, offset as usize);
                    bytes[4 - offset as usize..]
                        .copy_from_slice(&value_bytes[..4 - offset as usize]);
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

    fn read_u32_from_page(&mut self, addr: u32) -> Result<u32> {
        let page_id = self.storage.get_page_id(addr);
        if let Some(page) = self.storage.get_page(page_id) {
            if (addr as u64 + 3 - page.position as u64) >= PAGE_SIZE as u64 {
                let offset = addr & 0b11;

                let mut res_bytes: [u8; 4] = [0u8; 4];

                unsafe {
                    let lower_local_addr = addr - page.position;
                    let ptr: *const u8 = page.data.as_ptr().add(lower_local_addr as usize);
                    let bytes = slice::from_raw_parts(ptr, 4 - offset as usize);
                    res_bytes[..4 - offset as usize].copy_from_slice(&bytes[4 - offset as usize..]);

                    if let Some(upper_page) = self.storage.get_page(page_id + 1) {
                        let upper_local_addr = addr - upper_page.position;
                        let ptr: *const u8 =
                            upper_page.data.as_ptr().add(upper_local_addr as usize);
                        let bytes = slice::from_raw_parts(ptr, offset as usize);
                        res_bytes[4 - offset as usize..]
                            .copy_from_slice(&bytes[..4 - offset as usize]);
                    }

                    Ok(u32::from_le_bytes(res_bytes))
                }
            } else {
                unsafe {
                    let ptr = page
                        .data
                        .as_ptr()
                        .add(addr as usize - page.position as usize);
                    let bytes = ptr as *const [u8; 4];
                    Ok(u32::from_le_bytes(bytes.read()))
                }
            }
        } else {
            Ok(0)
        }
    }
}

pub struct RawPageMemory<T: RawPageStorage> {
    storage: T,
}

impl<T: RawPageStorage> Memory for RawPageMemory<T> {
    fn read_mem_u8(&mut self, addr: u32) -> Result<u8> {
        if let Some(page) = self.storage.get_page(self.storage.get_page_id(addr)) {
            let local_addr = (addr - page.position) as usize;
            Ok(page.data[local_addr])
        } else {
            Ok(0)
        }
    }

    fn read_mem_u32(&mut self, addr: u32) -> Result<u32> {
        self.read_u32_from_page(addr)
    }

    fn read_mem_u16(&mut self, addr: u32) -> Result<u16> {
        let offset_bits = (addr & 0b1) * 8;
        let full_value = self.read_mem_u32(addr & !(0b1))?;
        let u16_slice = (full_value >> (offset_bits)) & 0xffff;
        Ok(u16_slice as u16)
    }

    fn write_mem_u8(&mut self, addr: u32, value: u8) -> Result<()> {
        let page = self
            .storage
            .get_page_or_create(self.storage.get_page_id(addr));
        let local_addr = (addr - page.position) as usize;
        page.data[local_addr] = value;
        Ok(())
    }

    fn write_mem_u16(&mut self, addr: u32, value: u16) -> Result<()> {
        let offset_bits = (addr & 0b1) * 8;
        let full_value = self.read_mem_u32(addr & !(0b1))?;
        let cleared_value = !(0xFFFF << (offset_bits)) & full_value;
        let filled_value = cleared_value | ((value as u32) << (offset_bits));
        self.write_mem_u32(addr & !(0b1), filled_value)
    }

    fn write_mem_u32(&mut self, addr: u32, value: u32) -> Result<()> {
        self.write_u32_to_page(addr, value)
    }

    fn read_buf(&mut self, addr: u32, buf: &mut [u8]) -> Result<()> {
        for (i, byte) in buf.iter_mut().enumerate() {
            *byte = self.read_mem_u8(addr + i as u32)?;
        }
        Ok(())
    }

    fn write_buf(&mut self, addr: u32, buf: &[u8]) -> Result<()> {
        for (i, byte) in buf.iter().enumerate() {
            self.write_mem_u8(addr + i as u32, *byte)?;
        }
        Ok(())
    }
}
