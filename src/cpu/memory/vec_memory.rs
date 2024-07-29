use anyhow::{anyhow, Result};
use std::{fmt::Debug, fmt::Formatter};

use super::memory_core::{Memory, MEMORY_SIZE};

const PAGE_SIZE: u32 = 4096 * 16;

pub struct VecMemory {
    pages: Vec<(u32, Page)>,
}

impl Debug for VecMemory {
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

impl VecMemory {
    pub fn new() -> Self {
        VecMemory {
            pages: Vec::with_capacity(1024),
        }
    }
}

impl VecMemory {
    fn get_page_id(&self, addr: u32) -> u32 {
        addr / PAGE_SIZE
    }

    fn get_page(&self, page_id: u32) -> Option<&Page> {
        self.pages.iter().find(|p| p.0 == page_id).map(|p| &p.1)
    }

    fn get_page_mut(&mut self, page_id: u32) -> Option<&mut Page> {
        self.pages
            .iter_mut()
            .find(|p| p.0 == page_id)
            .map(|p| &mut p.1)
    }

    fn get_page_or_create(&mut self, page_id: u32) -> &mut Page {
        if let Some(i) = self.pages.iter_mut().position(|p| p.0 == page_id) {
            return &mut self.pages[i].1;
        }

        let position = page_id * PAGE_SIZE;
        let index = self.pages.len();
        self.pages.push((page_id, Page::new(position)));
        &mut self.pages[index].1
    }
}

impl Memory for VecMemory {
    fn read_mem_u8(&self, addr: u32) -> Result<u8> {
        let offset = addr & 0b11;
        let full_value = self.read_mem_u32(addr & !0b11)?;
        Ok(((full_value >> (offset * 8)) & (0xFF)) as u8)
    }

    fn read_mem_u32(&self, addr: u32) -> Result<u32> {
        let page_id = addr / PAGE_SIZE;
        if let Some(page) = self.get_page(page_id) {
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
                    let upper_page = self.get_page(upper_page_id);

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

    fn read_mem_u16(&self, addr: u32) -> Result<u16> {
        let offset_bits = (addr & 0b1) * 8;
        let full_value = self.read_mem_u32(addr & !(0b1))?;
        let u16_slice = (full_value >> (offset_bits)) & 0xffff;
        Ok(u16_slice as u16)
    }

    fn write_mem_u8(&mut self, addr: u32, value: u8) -> Result<()> {
        let offset = addr & 0b11;
        let full_value = self.read_mem_u32(addr & !(0b11))?;
        let masked_full_value = full_value & !(0xff << (8 * (offset)));
        let filled_full_value = masked_full_value | ((value as u32) << ((offset) * 8));
        self.write_mem_u32(addr & !(0b11), filled_full_value)
    }

    fn write_mem_u16(&mut self, addr: u32, value: u16) -> Result<()> {
        let offset_bits = (addr & 0b1) * 8;
        let full_value = self.read_mem_u32(addr & !(0b1))?;
        let cleared_value = !(0xFFFF << (offset_bits)) & full_value;
        let filled_value = cleared_value | ((value as u32) << (offset_bits));
        self.write_mem_u32(addr & !(0b1), filled_value)
    }

    fn write_mem_u32(&mut self, addr: u32, value: u32) -> Result<()> {
        let reordered_value = value.swap_bytes();

        let page_id = addr / PAGE_SIZE;
        if page_id > MEMORY_SIZE / PAGE_SIZE {
            return Err(anyhow!("Tried to access outside of memory bounds"));
        }

        let page = self.get_page_or_create(page_id);

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
                let upper_page = self.get_page_or_create(upper_page_id);

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
