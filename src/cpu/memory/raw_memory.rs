#[allow(unused_imports)]
use anyhow::{bail, Result};

use crate::cpu::cpu_core::{KERNEL_ADDR, KERNEL_SIZE};

use super::memory_core::Memory;

#[derive(Clone, Debug)]
pub struct ContinuousMemory {
    data: Vec<u8>,
    addr: u64,
}

impl ContinuousMemory {
    pub fn new(addr: u64, size: u64) -> Self {
        Self {
            data: vec![0; size as usize],
            addr,
        }
    }

    #[cfg(not(feature = "maxperf"))]
    fn check_bounds(&self, addr: u64, size: u64) -> Result<()> {
        if addr + size > self.data.len() as u64 {
            bail!("Out of bounds memory access at {}", addr);
        } else {
            Ok(())
        }
    }
}

impl Default for ContinuousMemory {
    fn default() -> Self {
        Self::new(KERNEL_ADDR, KERNEL_SIZE)
    }
}

impl Memory for ContinuousMemory {
    fn read_mem_u8(&mut self, addr: u64) -> Result<u8> {
        let addr = addr - self.addr;
        #[cfg(not(feature = "maxperf"))]
        self.check_bounds(addr, 1)?;
        unsafe { Ok(*self.data.as_ptr().add(addr as usize)) }
    }

    fn read_mem_u16(&mut self, addr: u64) -> Result<u16> {
        let addr = addr - self.addr;
        #[cfg(not(feature = "maxperf"))]
        self.check_bounds(addr, 2)?;
        unsafe {
            let ptr = self.data.as_ptr().add(addr as usize) as *const u16;
            Ok(ptr.read_unaligned())
        }
    }

    fn read_mem_u32(&mut self, addr: u64) -> Result<u32> {
        let addr = addr - self.addr;
        #[cfg(not(feature = "maxperf"))]
        self.check_bounds(addr, 4)?;
        unsafe {
            let ptr = self.data.as_ptr().add(addr as usize) as *const u32;
            Ok(ptr.read_unaligned())
        }
    }

    fn read_mem_u64(&mut self, addr: u64) -> Result<u64> {
        let addr = addr - self.addr;
        #[cfg(not(feature = "maxperf"))]
        self.check_bounds(addr, 8)?;
        unsafe {
            let ptr = self.data.as_ptr().add(addr as usize) as *const u64;
            Ok(ptr.read_unaligned())
        }
    }

    fn write_mem_u8(&mut self, addr: u64, value: u8) -> Result<()> {
        let addr = addr - self.addr;
        #[cfg(not(feature = "maxperf"))]
        self.check_bounds(addr, 1)?;
        unsafe {
            let ptr = self.data.as_mut_ptr().add(addr as usize);
            ptr.write(value);
        }
        Ok(())
    }

    fn write_mem_u16(&mut self, addr: u64, value: u16) -> Result<()> {
        let addr = addr - self.addr;
        #[cfg(not(feature = "maxperf"))]
        self.check_bounds(addr, 2)?;
        unsafe {
            let ptr = self.data.as_mut_ptr().add(addr as usize) as *mut u16;
            ptr.write_unaligned(value);
        }
        Ok(())
    }

    fn write_mem_u32(&mut self, addr: u64, value: u32) -> Result<()> {
        let addr = addr - self.addr;
        #[cfg(not(feature = "maxperf"))]
        self.check_bounds(addr, 4)?;
        unsafe {
            let ptr = self.data.as_mut_ptr().add(addr as usize) as *mut u32;
            ptr.write_unaligned(value);
        }
        Ok(())
    }

    fn write_mem_u64(&mut self, addr: u64, value: u64) -> Result<()> {
        let addr = addr - self.addr;
        #[cfg(not(feature = "maxperf"))]
        self.check_bounds(addr, 8)?;
        unsafe {
            let ptr = self.data.as_mut_ptr().add(addr as usize) as *mut u64;
            ptr.write_unaligned(value);
        }
        Ok(())
    }

    fn read_buf(&mut self, addr: u64, buf: &mut [u8]) -> Result<()> {
        let addr = addr - self.addr;
        #[cfg(not(feature = "maxperf"))]
        self.check_bounds(addr, buf.len() as u64)?;
        unsafe {
            let src = self.data.as_ptr().add(addr as usize);
            std::ptr::copy_nonoverlapping(src, buf.as_mut_ptr(), buf.len());
        }
        Ok(())
    }

    fn write_buf(&mut self, addr: u64, buf: &[u8]) -> Result<()> {
        let addr = addr - self.addr;
        #[cfg(not(feature = "maxperf"))]
        self.check_bounds(addr, buf.len() as u64)?;
        unsafe {
            let dst = self.data.as_mut_ptr().add(addr as usize);
            std::ptr::copy_nonoverlapping(buf.as_ptr(), dst, buf.len());
        }
        Ok(())
    }
}
