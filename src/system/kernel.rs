use std::fs::Metadata;

use anyhow::Result;

#[derive(Debug, PartialEq, Eq)]
pub enum SeekType {
    Start,
    Current,
    End,
}

impl From<u32> for SeekType {
    fn from(value: u32) -> Self {
        match value {
            0 => SeekType::Start,
            1 => SeekType::Current,
            2 => SeekType::End,
            _ => panic!("Invalid seek type"),
        }
    }
}

pub trait Kernel {
    fn new() -> Self;
    fn open_file(&mut self, path: &str) -> Result<u32>;
    fn read_fd(&mut self, fd: u32, buf: &mut [u8]) -> Result<usize>;
    fn write_fd(&mut self, fd: u32, buf: &[u8]) -> Result<usize>;
    fn close_fd(&mut self, fd: u32) -> Result<()>;
    fn create_file(&mut self, path: &str) -> Result<()>;
    fn seek_fd(&mut self, fd: u32, offset: usize, seek_type: SeekType) -> Result<u64>;
    fn fstat_fd(&mut self, fd: u32) -> Result<Metadata>;
    fn write_stderr(&mut self, buf: &[u8]);
    fn write_stdout(&mut self, buf: &[u8]);
    fn read_and_clear_stdout_buffer(&mut self) -> String;
}
