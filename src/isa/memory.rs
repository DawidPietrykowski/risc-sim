use std::{fmt::Debug, fmt::Formatter};

use anyhow::{anyhow, Result};

use rustc_hash::{FxBuildHasher, FxHashMap};

const PAGE_SIZE: u32 = 4096;

const MEMORY_SIZE: u32 = u32::MAX;

pub struct Memory {
    pages: FxHashMap<u32, Page>,
}

impl Debug for Memory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Memory {{ pages: {} }}", self.pages.len())
    }
}

struct Page {
    data: [u32; (PAGE_SIZE / 4) as usize],
    position: u32,
}

impl Page {
    pub fn new(position: u32) -> Page {
        Page {
            position,
            data: [0; (PAGE_SIZE / 4) as usize],
        }
    }
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            pages: FxHashMap::with_capacity_and_hasher(1024, FxBuildHasher::default()),
        }
    }

    pub fn read_mem_u8(&self, addr: u32) -> Result<u8> {
        let offset = addr & 0b11;
        let full_value = self.read_mem_u32(addr & !0b11)?;
        Ok(((full_value >> (offset * 8)) & (0xFFFF)) as u8)
    }

    pub fn read_mem_u32(&self, addr: u32) -> Result<u32> {
        let page_id = addr / PAGE_SIZE;
        if let Some(page) = self.pages.get(&page_id) {
            if 0 == addr & 3 {
                Ok(page.data[(addr - page.position) as usize / 4].swap_bytes())
            } else {
                let offset = addr & 0b11;

                let addr_lower = addr & !0b11;
                let val_lower: u32 = page.data[(addr_lower - page.position) as usize / 4];

                let addr_upper = addr_lower + 4;

                let val_upper;
                if addr_upper >= (page.position + PAGE_SIZE) {
                    let upper_page_id = page_id + 1;
                    let upper_page = self // TODO: dont allocate new if not found
                        .pages
                        .get(&upper_page_id);

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

    pub fn read_mem_u16(&self, addr: u32) -> Result<u16> {
        let offset_bits = (addr & 0b1) * 8;
        let full_value = self.read_mem_u32(addr & !(0b1))?;
        let u16_slice = (full_value >> (offset_bits)) & 0xffffffff;
        Ok(u16_slice as u16)
    }

    pub fn write_mem_u8(&mut self, addr: u32, value: u8) -> Result<()> {
        let offset = addr & 0b11;
        let full_value = self.read_mem_u32(addr & !(0b11))?;
        let masked_full_value = full_value & !(0xff << (8 * (offset)));
        let filled_full_value = masked_full_value | ((value as u32) << ((offset) * 8));
        self.write_mem_u32(addr & !(0b11), filled_full_value)
    }

    pub fn write_mem_u16(&mut self, addr: u32, value: u16) -> Result<()> {
        let offset_bits = (addr & 0b1) * 8;
        let full_value = self.read_mem_u32(addr & !(0b1))?;
        let cleared_value = !(0xFFFFFFFF << (offset_bits)) & full_value;
        let filled_value = cleared_value | ((value as u32) << (offset_bits));
        self.write_mem_u32(addr & !(0b1), filled_value)
    }

    pub fn write_mem_u32(&mut self, addr: u32, value: u32) -> Result<()> {
        let reordered_value = value.swap_bytes();

        let page_id = addr / PAGE_SIZE;
        if page_id > MEMORY_SIZE / PAGE_SIZE {
            return Err(anyhow!("Tried to access outside of memory bounds"));
        }

        let page = self
            .pages
            .entry(page_id)
            .or_insert_with(|| Page::new(page_id * PAGE_SIZE));

        if addr & 0b11 == 0 {
            page.data[(addr - page.position) as usize / 4] = reordered_value;
        } else {
            let addr_lower: u32 = addr & !0b11;
            let addr_upper = addr_lower + 4;
            let offset = addr & 0b11;
            page.data[(addr_lower - page.position) as usize / 4] &= !(u32::MAX >> (8 * offset));
            page.data[(addr_lower - page.position) as usize / 4] |= reordered_value >> (8 * offset);

            if addr_upper >= (page.position + PAGE_SIZE) {
                let upper_page_id = page_id + 1;
                let upper_page = self
                    .pages
                    .entry(upper_page_id)
                    .or_insert_with(|| Page::new(upper_page_id * PAGE_SIZE));

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
}
