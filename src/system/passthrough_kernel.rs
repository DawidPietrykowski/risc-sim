use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
};

use super::kernel::{Kernel, SeekType};
use anyhow::{anyhow, Context, Result};

const STDOUT_BUFFER_SIZE: usize = 1024 * 32;

pub struct PassthroughKernel {
    fd_map: HashMap<u32, File>,
    next_id: u32,
    pub stdout_buffer: Vec<u8>,
    pub stdin_buffer: Vec<u8>,
    pub stderr_buffer: Vec<u8>,
}

impl Default for PassthroughKernel {
    fn default() -> Self {
        let stdout_buffer = Vec::<u8>::with_capacity(STDOUT_BUFFER_SIZE);
        Self {
            fd_map: HashMap::new(),
            next_id: 3,
            stdout_buffer,
            stdin_buffer: Vec::new(),
            stderr_buffer: Vec::new(),
        }
    }
}

impl PassthroughKernel {
    fn get_file(&mut self, fd: u32) -> Result<&mut File> {
        self.fd_map
            .get_mut(&fd)
            .ok_or_else(|| anyhow!("Invalid fd"))
    }
}

impl Kernel for PassthroughKernel {
    fn new () -> Self {
        Self::default()
    }
    
    fn open_file(&mut self, path: &str) -> Result<u32> {
        let file = File::open(path)?;
        let id = self.next_id;
        self.fd_map.insert(id, file);
        self.next_id += 1;
        Ok(id)
    }

    fn read_fd(&mut self, fd: u32, buf: &mut [u8]) -> Result<usize> {
        self.get_file(fd)?.read(buf).context("Failed to read file")
    }

    fn write_fd(&mut self, fd: u32, buf: &[u8]) -> Result<usize> {
        self.get_file(fd)?
            .write(buf)
            .context("Failed to write file")
    }

    fn close_fd(&mut self, fd: u32) -> Result<()> {
        self.fd_map.remove(&fd).context("Invalid fd")?;
        Ok(())
    }

    fn create_file(&mut self, path: &str) -> Result<()> {
        File::create(path).context("Failed to create file")?;
        Ok(())
    }

    fn seek_fd(&mut self, fd: u32, offset: usize, seek_type: SeekType) -> Result<u64> {
        let seek = match seek_type {
            SeekType::Start => SeekFrom::Start(offset as u64),
            SeekType::Current => SeekFrom::Current(offset as i64),
            SeekType::End => SeekFrom::End(offset as i64),
        };
        self.get_file(fd)?.seek(seek).context("Failed to seek file")
    }

    fn fstat_fd(&mut self, fd: u32) -> Result<std::fs::Metadata> {
        self.get_file(fd)?.metadata().context("Failed to stat file")
    }

    fn write_stdout(&mut self, buf: &[u8]) {
        self.stdout_buffer.extend(buf);
        print!("{}", String::from_utf8_lossy(buf));
    }

    fn write_stderr(&mut self, buf: &[u8]) {
        self.stderr_buffer.extend(buf);
        print!("{}", String::from_utf8_lossy(buf));
    }

    fn read_and_clear_stdout_buffer(&mut self) -> String {
        let stdout_buffer = String::from_utf8(self.stdout_buffer.clone()).unwrap();
        self.stdout_buffer.clear();
        stdout_buffer
    }
}
