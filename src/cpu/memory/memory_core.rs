pub const MEMORY_SIZE: u32 = u32::MAX;
pub const MEMORY_CAPACITY: usize = 48;
use std::fmt::Debug;

use anyhow::Result;

pub trait Memory: Debug {
    fn read_mem_u8(&mut self, addr: u32) -> Result<u8>;
    fn read_mem_u32(&mut self, addr: u32) -> Result<u32>;
    fn read_mem_u16(&mut self, addr: u32) -> Result<u16>;
    fn write_mem_u8(&mut self, addr: u32, value: u8) -> Result<()>;
    fn write_mem_u16(&mut self, addr: u32, value: u16) -> Result<()>;
    fn write_mem_u32(&mut self, addr: u32, value: u32) -> Result<()>;
    fn read_buf(&mut self, addr: u32, buf: &mut [u8]) -> Result<()>;
    fn write_buf(&mut self, addr: u32, buf: &[u8]) -> Result<()>;
}
