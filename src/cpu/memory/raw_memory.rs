use anyhow::{bail, Result};

use crate::cpu::cpu_core::{KERNEL_ADDR, KERNEL_SIZE};

use super::memory_core::Memory;

#[derive(Debug)]
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
        Ok(self.data[addr as usize])
    }

    fn read_mem_u16(&mut self, addr: u64) -> Result<u16> {
        let addr = addr - self.addr;
        #[cfg(not(feature = "maxperf"))]
        self.check_bounds(addr, 2)?;
        let bytes = &self.data[addr as usize..addr as usize + 2];
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn read_mem_u32(&mut self, addr: u64) -> Result<u32> {
        let addr = addr - self.addr;
        #[cfg(not(feature = "maxperf"))]
        self.check_bounds(addr, 4)?;
        let bytes = &self.data[addr as usize..addr as usize + 4];
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_mem_u64(&mut self, addr: u64) -> Result<u64> {
        let addr = addr - self.addr;
        #[cfg(not(feature = "maxperf"))]
        self.check_bounds(addr, 8)?;
        let bytes = &self.data[addr as usize..addr as usize + 8];
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn write_mem_u8(&mut self, addr: u64, value: u8) -> Result<()> {
        let addr = addr - self.addr;
        #[cfg(not(feature = "maxperf"))]
        self.check_bounds(addr, 1)?;
        self.data[addr as usize] = value;
        Ok(())
    }

    fn write_mem_u16(&mut self, addr: u64, value: u16) -> Result<()> {
        let addr = addr - self.addr;
        #[cfg(not(feature = "maxperf"))]
        self.check_bounds(addr, 2)?;
        let bytes = value.to_le_bytes();
        self.data[addr as usize..addr as usize + 2].copy_from_slice(&bytes);
        Ok(())
    }

    fn write_mem_u32(&mut self, addr: u64, value: u32) -> Result<()> {
        let addr = addr - self.addr;
        #[cfg(not(feature = "maxperf"))]
        self.check_bounds(addr, 4)?;
        let bytes = value.to_le_bytes();
        self.data[addr as usize..addr as usize + 4].copy_from_slice(&bytes);
        Ok(())
    }

    fn write_mem_u64(&mut self, addr: u64, value: u64) -> Result<()> {
        let addr = addr - self.addr;
        #[cfg(not(feature = "maxperf"))]
        self.check_bounds(addr, 8)?;
        let bytes = value.to_le_bytes();
        self.data[addr as usize..addr as usize + 8].copy_from_slice(&bytes);
        Ok(())
    }

    fn read_buf(&mut self, addr: u64, buf: &mut [u8]) -> Result<()> {
        let addr = addr - self.addr;
        #[cfg(not(feature = "maxperf"))]
        self.check_bounds(addr, buf.len() as u64)?;
        buf.copy_from_slice(&self.data[addr as usize..addr as usize + buf.len()]);
        Ok(())
    }

    fn write_buf(&mut self, addr: u64, buf: &[u8]) -> Result<()> {
        let addr = addr - self.addr;
        #[cfg(not(feature = "maxperf"))]
        self.check_bounds(addr, buf.len() as u64)?;
        self.data[addr as usize..addr as usize + buf.len()].copy_from_slice(buf);
        Ok(())
    }
}
