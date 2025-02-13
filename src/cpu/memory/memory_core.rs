pub const MEMORY_SIZE: u64 = 0x100000;
pub const MEMORY_CAPACITY: usize = 48;
use std::fmt::Debug;

use anyhow::Result;

pub trait Memory: Debug {
    fn read_mem_u8(&mut self, addr: u64) -> Result<u8>;
    fn read_mem_u16(&mut self, addr: u64) -> Result<u16>;
    fn read_mem_u32(&mut self, addr: u64) -> Result<u32>;
    fn read_mem_u64(&mut self, addr: u64) -> Result<u64>;
    fn write_mem_u8(&mut self, addr: u64, value: u8) -> Result<()>;
    fn write_mem_u16(&mut self, addr: u64, value: u16) -> Result<()>;
    fn write_mem_u32(&mut self, addr: u64, value: u32) -> Result<()>;
    fn write_mem_u64(&mut self, addr: u64, value: u64) -> Result<()>;
    fn read_buf(&mut self, addr: u64, buf: &mut [u8]) -> Result<()>;
    fn write_buf(&mut self, addr: u64, buf: &[u8]) -> Result<()>;
}
