use anyhow::{anyhow, Ok, Result};
use std::{cmp::min, fmt::{Debug, Formatter}};

use super::memory_core::{Memory, MEMORY_SIZE};

pub const PAGE_SIZE: u32 = 4096 * 256;

const CACHE_SIZE: u32 = 64;

#[derive(Clone, Copy)]
struct CacheEntry {
    addr: u32,
    data: u32
}

struct MemoryCache {
    entries: [CacheEntry; CACHE_SIZE as usize],
    next_id: u32,
    count: u32,
}

#[allow(unused)]
pub trait PageStorage {
    fn new() -> Self;
    fn get_page_id(&self, addr: u32) -> u32;
    fn get_page(&self, page_id: u32) -> Option<&Page>;
    fn get_page_mut(&mut self, page_id: u32) -> Option<&mut Page>;
    fn get_page_or_create(&mut self, page_id: u32) -> &mut Page;
    fn len(&self) -> usize;
}

impl<T: PageStorage> Debug for PageMemory<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Memory {{ pages: {} }}", self.storage.len())
    }
}

pub struct Page {
    pub data: Box<[u32; (PAGE_SIZE / 4) as usize]>,
    pub position: u32,
}

impl Page {
    pub fn new(position: u32) -> Page {
        Page {
            position,
            data: Box::new([0; (PAGE_SIZE / 4) as usize]),
        }
    }
}

impl<T: PageStorage> PageMemory<T> {
    pub fn new() -> Self {
        #[cfg(feature = "memcache")]
        return PageMemory { storage: T::new() ,
            cache: MemoryCache {
                entries: [CacheEntry{addr: 0, data: 0}; CACHE_SIZE as usize],
                next_id: 0,
                count: 0,
            }
        };
        #[cfg(not(feature = "memcache"))]
        PageMemory { storage: T::new() }
    }



    #[cfg(feature = "memcache")]
    fn get(&self, addr: u32) -> Option<u32> {
        // assert!(self.count <= CACHE_SIZE);
        // assert!(self.next_id < CACHE_SIZE);
        // println!("Cache get: {:?}, next_id: {}, count: {}", addr, self.next_id, self.count);
        // for entry in self.entries.iter().take(self.count as usize) {
        //     if entry.addr == addr {
        //         return Some(entry.data);
        //     }
        // }

        for i in 0..self.cache.count {
            if self.cache.entries[i as usize].addr == addr {
                return Some(self.cache.entries[i as usize].data);
            }
        }

        None
    }

    #[cfg(feature = "memcache")]
    fn set(&mut self, addr: u32, data: u32)
     {
        // println!("Cache set: {:?}, next_id: {}, count: {}", addr, self.next_id, self.count);
        // self.entries[self.next_id as usize] = CacheEntry { addr, data };
        // self.next_id = (self.next_id + 1) % CACHE_SIZE;
        // self.count = min(CACHE_SIZE, self.count + 1);

        // assert!(self.count <= CACHE_SIZE);
        // assert!(self.next_id < CACHE_SIZE);

        for i in 0..self.cache.count {
            let existing_entry = self.cache.entries[i as usize];
            if existing_entry.addr == addr {
                // self.write_u32_to_page(existing_entry.addr, existing_entry.data).unwrap();
                self.cache.entries[i as usize].data = data;
                return;
            }
        }

        if self.cache.next_id < self.cache.count{
            let entry = self.cache.entries[self.cache.next_id as usize];
            self.write_u32_to_page(entry.addr, entry.data).unwrap();
        }
        self.cache.entries[self.cache.next_id as usize] = CacheEntry { addr, data };
        self.cache.next_id = (self.cache.next_id + 1) % CACHE_SIZE;
        self.cache.count = min(CACHE_SIZE, self.cache.count + 1);
    }

    fn write_u32_to_page(&mut self, addr: u32, value: u32) -> Result<()> {
        let reordered_value = value.swap_bytes();

        let page_id = addr / PAGE_SIZE;
        if page_id > MEMORY_SIZE / PAGE_SIZE {
            return Err(anyhow!("Tried to access outside of memory bounds"));
        }

        let page = self.storage.get_page_or_create(page_id);

        if addr & 0b11 == 0 {
            page.data[(addr - page.position) as usize / 4] = reordered_value;
        } else {
            let addr_lower: u32 = addr & !0b11;
            let addr_upper = addr_lower + 4;
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

    fn read_u32_from_page(&self, addr: u32) -> Result<u32> {
        let page_id = addr / PAGE_SIZE;
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

pub struct PageMemory<T: PageStorage> {
    pub storage: T,
    #[cfg(feature = "memcache")]
    cache: MemoryCache
}

impl<T: PageStorage> Memory for PageMemory<T> {
    fn read_mem_u8(&mut self, addr: u32) -> Result<u8> {
        let offset = addr & 0b11;
        let full_value = self.read_mem_u32(addr & !0b11)?;
        Ok(((full_value >> (offset * 8)) & (0xFF)) as u8)
    }

    fn read_mem_u32(&mut self, addr: u32) -> Result<u32> {
        #[cfg(feature = "memcache")]
        if let Some(res) = self.get(addr) {
            return Ok(res);
        }
        let res = self.read_u32_from_page(addr)?;
        #[cfg(feature = "memcache")]
        self.set(addr, res);
        Ok(res)
    }

    fn read_mem_u16(&mut self, addr: u32) -> Result<u16> {
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
        #[cfg(feature = "memcache")]{
        self.set(addr, value);
    Ok(())}
        #[cfg(not(feature = "memcache"))]
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
