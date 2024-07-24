use std::{fmt::Debug, fmt::Formatter};

use anyhow::{anyhow, Result};

use rustc_hash::{FxBuildHasher, FxHashMap};

const PAGE_SIZE: u32 = 4096 * 16;

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
        // todo!()
        let offset = addr & 0b11;
        let full_value = self.read_mem_u32(addr & !(0b11))?;
        // println!("full_value: {:#x}", full_value);
        // println!("offset: {:#x}", offset);
        // println!("masked_full_value: {:#x}", masked_full_value);
        let masked_full_value = full_value & (0xff << (8 * (3 - offset)));
        Ok(((masked_full_value) >> ((3 - offset) * 8)) as u8)
    }

    pub fn read_mem_u32(&self, addr: u32) -> Result<u32> {
        let page_id = addr / PAGE_SIZE;
        if let Some(page) = self.pages.get(&page_id) {
            // Ok(page.data[(addr - page.position) as usize])
            if 0 == addr & 3 {
                Ok(page.data[(addr - page.position) as usize / 4])
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
                        val_upper = upper_page.data[(addr_upper - upper_page.position) as usize / 4];
                    } else {
                        val_upper = 0;
                    }
    
                } else {
                    val_upper = page.data[(addr_upper - page.position) as usize / 4];
                }



                let mut res: u32 = 0;
                res |= val_lower << (8 * offset);
                res |= val_upper >> (8 * (4 - offset));
                Ok(res)
            }
        } else {
            Ok(0)
        }
    }

    pub fn read_mem_u16(&self, addr: u32) -> Result<u16> {
        // Ok((self.read_mem_u8(addr)? as u16) | (self.read_mem_u8(addr + 1)? as u16) << 8)
        // todo!()
        // let val = self.read_mem_u32(addr & !11)?;
        // if addr & 1 == 0 {
        //     Ok(val as u16)
        // } else {
        //     Ok((val >> 16) as u16)
        // }

        let offset = (addr & 0b10) >> 1;
        let full_value = self.read_mem_u32(addr & !(0b10))?;
        let masked_full_value = full_value & !(0xffff << (16 * (offset)));

        println!("full_value: {:#x}", full_value);
        println!("offset: {:#x}", offset);
        println!("masked_full_value: {:#x}", masked_full_value);

        Ok(((masked_full_value as u32) >> ((1 - offset) * 16)) as u16)
    }

    pub fn write_mem_u8(&mut self, addr: u32, value: u8) -> Result<()> {
        // todo!();
        // let page_id: u32 = addr / PAGE_SIZE;
        // if page_id > MEMORY_SIZE / PAGE_SIZE {
        //     return Err(anyhow!("Tried to access outside of memory bounds"));
        // }

        // self.pages
        //     .entry(page_id)
        //     .or_insert_with(|| Page::new(page_id * PAGE_SIZE));

        // let page = self
        //     .pages
        //     .get_mut(&(addr / PAGE_SIZE))
        //     .context("Failed to allocate new page")?;

        // page.data[(addr - page.position) as usize] = value;

        // Ok(())

        let offset = addr & 0b11;
        let full_value = self.read_mem_u32(addr & !(0b11))?;
        let masked_full_value = full_value & !(0xff << (8 * (3 - offset)));
        let filled_full_value = masked_full_value | ((value as u32) << ((3 - offset) * 8));
        self.write_mem_u32(addr & !(0b11), filled_full_value)
    }

    pub fn write_mem_u16(&mut self, addr: u32, value: u16) -> Result<()> {
        // todo!();
        // let val = self.read_mem_u16(addr)?;
        // if addr & 1 == 0 {

        let offset = (addr & 0b10) >> 1;
        let full_value = self.read_mem_u32(addr & !(0b10))?;
        let masked_full_value = full_value & !(0xffff << (16 * (1 - offset)));
        let filled_full_value = masked_full_value | ((value as u32) << ((1 - offset) * 16));
        self.write_mem_u32(addr & !(0b10), filled_full_value)
    }

    pub fn write_mem_u32(&mut self, addr: u32, value: u32) -> Result<()> {
        // self.write_mem_u16(addr, value as u16)?;
        // self.write_mem_u16(addr + 2, (value >> 16) as u16)?;
        // Ok(())

        let page_id = addr / PAGE_SIZE;
        if page_id > MEMORY_SIZE / PAGE_SIZE {
            return Err(anyhow!("Tried to access outside of memory bounds"));
        }

        let page = self
            .pages
            .entry(page_id)
            .or_insert_with(|| Page::new(page_id * PAGE_SIZE));

        // let page = self
        //     .pages
        //     .get_mut(&(addr / PAGE_SIZE))
        //     .context("Failed to allocate new page")?;

        if addr & 0b11 == 0 {
            page.data[(addr - page.position) as usize / 4] = value;
        } else {
            let addr_lower: u32 = addr & !0b11;
            let addr_upper = addr_lower + 4;
            let offset = addr & 0b11;
            page.data[(addr_lower - page.position) as usize / 4] &= !(u32::MAX >> (8 * offset));
            page.data[(addr_lower - page.position) as usize / 4] |= value >> (8 * offset);

            if addr_upper >= (page.position + PAGE_SIZE) {
                let upper_page_id = page_id + 1;
                let upper_page = self
                    .pages
                    .entry(upper_page_id)
                    .or_insert_with(|| Page::new(upper_page_id * PAGE_SIZE));

                upper_page.data[(addr_upper - upper_page.position) as usize / 4] &=
                    !(u32::MAX << (8 * (4 - offset)));
                upper_page.data[(addr_upper - upper_page.position) as usize / 4] |= value << (8 * (4 - offset));
            } else {
                page.data[(addr_upper - page.position) as usize / 4] &=
                    !(u32::MAX << (8 * (4 - offset)));
                page.data[(addr_upper - page.position) as usize / 4] |= value << (8 * (4 - offset));
            }
        }

        Ok(())
    }
}
