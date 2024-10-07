use anyhow::{Ok, Result};
use std::fmt::{Debug, Formatter};

use super::memory_core::Memory;

pub(crate) const PAGE_SIZE_LOG2: u32 = 18;
pub const PAGE_SIZE: u64 = 1 << PAGE_SIZE_LOG2;

#[allow(unused)]
#[allow(clippy::len_without_is_empty)]
pub trait PageStorage {
    fn new() -> Self;
    fn get_page_id(&self, addr: u64) -> u64;
    fn get_page(&self, page_id: u64) -> Option<&Page>;
    fn get_page_mut(&mut self, page_id: u64) -> Option<&mut Page>;
    fn get_page_or_create(&mut self, page_id: u64) -> &mut Page;
    fn len(&self) -> usize;
}

impl<T: PageStorage> Debug for PageMemory<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Memory {{ pages: {} }}", self.storage.len())
    }
}

#[derive(Clone)]
pub struct Page {
    pub data: Box<[u32; (PAGE_SIZE / 4) as usize]>,
    pub position: u64,
}

impl Page {
    pub fn new(position: u64) -> Page {
        Page {
            position,
            data: Box::new([0; (PAGE_SIZE / 4) as usize]),
        }
    }
}

impl<T: PageStorage> PageMemory<T> {
    pub fn new() -> Self {
        PageMemory { storage: T::new() }
    }

    fn write_u32_to_page(&mut self, addr: u64, value: u32) -> Result<()> {
        let reordered_value = value.swap_bytes();

        let page_id = self.storage.get_page_id(addr);

        let page = self.storage.get_page_or_create(page_id);

        if addr & 0b11 == 0 {
            page.data[(addr - page.position) as usize / 4] = reordered_value;
        } else {
            let addr_lower: u64 = addr & !0b11;
            let addr_upper: u64 = addr_lower + 4;
            let offset = addr & 0b11;
            page.data[(addr_lower - page.position) as usize / 4] &= !(u32::MAX >> (8 * offset));
            page.data[(addr_lower - page.position) as usize / 4] |= reordered_value >> (8 * offset);

            if (addr_upper - page.position) >= PAGE_SIZE {
                let upper_page_id = page_id + 1;
                let upper_page = self.storage.get_page_or_create(upper_page_id);

                upper_page.data[(addr_upper - upper_page.position) as usize / 4] &=
                    !(u32::MAX << (8 * (4 - offset)));
                upper_page.data[(addr_upper - upper_page.position) as usize / 4] |=
                    reordered_value << (8 * (4 - offset));
            } else {
                page.data[(addr_upper - page.position) as usize / 4] &=
                    !(u32::MAX << (8 * (4 - offset)));
                page.data[(addr_upper - page.position) as usize / 4] |=
                    reordered_value << (8 * (4 - offset));
            }
        }

        Ok(())
    }

    fn read_u32_from_page(&self, addr: u64) -> Result<u32> {
        let page_id = self.storage.get_page_id(addr);
        if let Some(page) = self.storage.get_page(page_id) {
            if 0 == addr & 3 {
                Ok(page.data[(addr - page.position) as usize / 4].swap_bytes())
            } else {
                let offset = addr & 0b11;

                let addr_lower = addr & !0b11;
                let val_lower: u32 = page.data[(addr_lower - page.position) as usize / 4];

                let addr_upper = addr_lower + 4;

                let val_upper;
                if (addr_upper - page.position) >= PAGE_SIZE {
                    let upper_page_id = page_id + 1;
                    let upper_page = self.storage.get_page(upper_page_id);

                    if let Some(upper_page) = upper_page {
                        val_upper =
                            upper_page.data[(addr_upper - upper_page.position) as usize / 4];
                    } else {
                        val_upper = 0;
                    }
                } else {
                    val_upper = page.data[(addr_upper - page.position) as usize / 4];
                }

                let mut res: u32 = 0;
                res |= val_lower << (8 * offset);
                res |= val_upper >> (8 * (4 - offset));
                Ok(res.swap_bytes())
            }
        } else {
            Ok(0)
        }
    }
}

#[derive(Clone, Copy)]
pub struct PageMemory<T: PageStorage> {
    storage: T,
}

impl<T: PageStorage> Memory for PageMemory<T> {
    fn read_mem_u8(&mut self, addr: u64) -> Result<u8> {
        let offset = addr & 0b11;
        let full_value = self.read_mem_u32(addr & !0b11)?;
        Ok(((full_value >> (offset * 8)) & (0xFF)) as u8)
    }

    fn read_mem_u32(&mut self, addr: u64) -> Result<u32> {
        self.read_u32_from_page(addr)
    }

    fn read_mem_u16(&mut self, addr: u64) -> Result<u16> {
        let offset_bits = (addr & 0b1) * 8;
        let full_value = self.read_mem_u32(addr & !(0b1))?;
        let u16_slice = (full_value >> (offset_bits)) & 0xffff;
        Ok(u16_slice as u16)
    }

    fn write_mem_u8(&mut self, addr: u64, value: u8) -> Result<()> {
        let offset = addr & 0b11;
        let full_value = self.read_mem_u32(addr & !(0b11))?;
        let masked_full_value = full_value & !(0xff << (8 * (offset)));
        let filled_full_value = masked_full_value | ((value as u32) << ((offset) * 8));
        self.write_mem_u32(addr & !(0b11), filled_full_value)
    }

    fn write_mem_u16(&mut self, addr: u64, value: u16) -> Result<()> {
        let offset_bits = (addr & 0b1) * 8;
        let full_value = self.read_mem_u32(addr & !(0b1))?;
        let cleared_value = !(0xFFFF << (offset_bits)) & full_value;
        let filled_value = cleared_value | ((value as u32) << (offset_bits));
        self.write_mem_u32(addr & !(0b1), filled_value)
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
        let lower_value = self.read_mem_u32(addr)?;
        let upper_value = self.read_mem_u32(addr + 4)?;
        Ok((upper_value as u64) << 32 | lower_value as u64)
    }

    fn write_mem_u64(&mut self, addr: u64, value: u64) -> Result<()> {
        self.write_mem_u32(addr, value as u32)?;
        self.write_mem_u32(addr + 4, (value >> 32) as u32)
    }
}
