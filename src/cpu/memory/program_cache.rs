use crate::{
    cpu::cpu_core::CpuMode,
    types::{decode_program_line, ProgramLine, Word},
};

use super::memory_core::Memory;

use anyhow::Result;

pub struct ProgramCache {
    start_addr: u64,
    end_addr: u64,
    data: Vec<ProgramLine>,
}

impl ProgramCache {
    pub fn empty() -> ProgramCache {
        ProgramCache {
            start_addr: 0,
            end_addr: 0,
            data: Vec::new(),
        }
    }

    pub fn new(
        start_addr: u64,
        end_addr: u64,
        memory: &mut dyn Memory,
        mode: CpuMode,
    ) -> Result<ProgramCache> {
        let mut data = Vec::new();
        for i in (start_addr..end_addr).step_by(4) {
            data.push(decode_program_line(Word(memory.read_mem_u32(i)?), mode)?);
        }
        Ok(ProgramCache {
            start_addr,
            end_addr,
            data,
        })
    }

    pub fn try_get_line(&self, addr: u64) -> Option<ProgramLine> {
        if addr < self.start_addr || addr >= self.end_addr {
            return None;
        }
        Some(self.data[((addr - self.start_addr) / 4) as usize])
    }

    #[inline(always)]
    pub fn get_line_unchecked(&self, addr: u64) -> ProgramLine {
        unsafe {
            *self
                .data
                .get_unchecked(((addr - self.start_addr) / 4) as usize)
        }
    }
}
