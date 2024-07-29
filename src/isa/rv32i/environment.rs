use std::{
    mem,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{isa, types::*};

use anyhow::Context;
use nix::time::{clock_gettime, ClockId};

#[repr(C)]
pub struct Stat {
    pub dev: u64,
    pub ino: u64,
    pub mode: u32,
    pub nlink: u32,
    pub uid: u32,
    pub gid: u32,
    pub rdev: u64,
    pub size: i64,
    pub blksize: i64,
    pub blocks: i64,
    pub atime: u64,
    pub mtime: u64,
    pub ctime: u64,
}

impl Stat {
    pub fn new_stdout(buffer_size: u32) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Stat {
            dev: 1,
            ino: 1,
            mode: 0o020666,
            nlink: 1,
            uid: 1000,
            gid: 1000,
            rdev: 1,
            size: buffer_size as i64,
            blksize: 1024,
            blocks: (buffer_size as i64 + 511) / 512,
            atime: now,
            mtime: now,
            ctime: now,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let bytes_ptr: *const u8 = self as *const isa::rv32i::environment::Stat as *const u8;
            Vec::from(std::slice::from_raw_parts(
                bytes_ptr,
                mem::size_of::<Stat>(),
            ))
        }
    }
}

pub const RV32I_SET_E: [Instruction; 2] = [
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC12_MASK,
        bits: 0b0 << FUNC12_POS | 0b1110011,
        name: "ECALL",
        instruction_type: InstructionType::I,
        operation: |cpu, _word| {
            let syscall_num = cpu.read_x_u32(ABIRegister::A(7).to_x_reg_id() as u8)?;
            match syscall_num {
                57 => {
                    // Close syscall
                    let fd = cpu.read_x_u32(ABIRegister::A(0).to_x_reg_id() as u8)?;
                    cpu.debug_print(|| format!("close: {}", fd));
                    cpu.write_x_u32(ABIRegister::A(0).to_x_reg_id() as u8, 0)?;
                }
                64 => {
                    // Write syscall
                    let fd = cpu.read_x_u32(ABIRegister::A(0).to_x_reg_id() as u8)?;
                    let buffer_addr = cpu.read_x_u32(ABIRegister::A(1).to_x_reg_id() as u8)?;
                    let len = cpu.read_x_u32(ABIRegister::A(2).to_x_reg_id() as u8)?;

                    if fd != 1 {
                        todo!()
                    }

                    for i in 0..len {
                        let byte = cpu.read_mem_u8(buffer_addr + i)?;
                        cpu.push_stdout(byte);
                    }
                    cpu.write_x_u32(ABIRegister::A(0).to_x_reg_id() as u8, len)?;

                    cpu.debug_print(|| format!("write: {} {:#x} {}", fd, buffer_addr, len));
                    cpu.debug_print(|| {
                        format!("written: {}", String::from_utf8_lossy(&cpu.stdout_buffer))
                    });
                }
                80 => {
                    // fstat
                    let fd = cpu.read_x_u32(ABIRegister::A(0).to_x_reg_id() as u8)?;
                    let stat_addr = cpu.read_x_u32(ABIRegister::A(1).to_x_reg_id() as u8)?;
                    cpu.debug_print(|| format!("fstat: {} addr: {:#x}", fd, stat_addr));
                    if fd == 1 {
                        Stat::new_stdout(cpu.stdout_buffer.len() as u32)
                            .to_bytes()
                            .iter()
                            .enumerate()
                            .for_each(|(i, b)| {
                                cpu.write_mem_u8(stat_addr + i as u32, *b).unwrap();
                            });
                        cpu.write_x_u32(ABIRegister::A(0).to_x_reg_id() as u8, 0)?;
                    } else {
                        todo!()
                    }
                }
                93 => {
                    // Exit syscall
                    cpu.set_halted();
                    cpu.write_x_u32(ABIRegister::A(0).to_x_reg_id() as u8, 0)?;
                }
                214 => {
                    // brk
                    let addr = cpu.read_x_u32(ABIRegister::A(0).to_x_reg_id() as u8)?;
                    cpu.debug_print(|| format!("brk call: {:#x}", addr));
                    if addr != 0 {
                        cpu.program_brk = addr;
                    }
                    cpu.write_x_u32(ABIRegister::A(0).to_x_reg_id() as u8, cpu.program_brk)?;
                    cpu.debug_print(|| format!("brk: {:#x}", cpu.program_brk));
                }
                403 => {
                    // clock_gettime

                    let clock_id = cpu.read_x_u32(ABIRegister::A(0).to_x_reg_id() as u8)?;
                    let timespec_addr = cpu.read_x_u32(ABIRegister::A(1).to_x_reg_id() as u8)?;

                    let now = clock_gettime(ClockId::from_raw(clock_id.try_into().unwrap()))
                        .context("clock_gettime")?;

                    let seconds = now.tv_sec() as u32;
                    let nanos = now.tv_nsec() as u32;

                    cpu.write_mem_u32(timespec_addr, seconds)?;
                    cpu.write_mem_u32(timespec_addr + 4, nanos)?;

                    cpu.write_x_u32(ABIRegister::A(0).to_x_reg_id() as u8, 0)?;

                    cpu.debug_print(|| format!("clock_gettime: {} {}", seconds, nanos));
                }
                code => {
                    todo!("Unsupported syscall: {}", code)
                }
            }
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC12_MASK,
        bits: 0b1 << FUNC12_POS | 0b1110011,
        name: "EBREAK",
        instruction_type: InstructionType::I,
        operation: |_cpu, _word| {
            todo!();
        },
    },
];
