use std::{
    fs::Metadata, mem, os::unix::fs::MetadataExt, time::{SystemTime, UNIX_EPOCH}
};

use crate::{isa, system::kernel::{Kernel, SeekType}, types::*};

use anyhow::{bail, Context, Result};
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

impl From<Metadata> for Stat {
    fn from(metadata: Metadata) -> Self {
        Stat {
            dev: metadata.dev(),
            ino: metadata.ino(),
            mode: metadata.mode() as u32,
            nlink: metadata.nlink() as u32,
            uid: metadata.uid(),
            gid: metadata.gid(),
            rdev: metadata.rdev(),
            size: metadata.len() as i64,
            blksize: metadata.blksize() as i64,
            blocks: (metadata.len() as i64 + 511) / 512,
            atime: metadata.atime() as u64,
            mtime: metadata.mtime() as u64,
            ctime: metadata.ctime() as u64,
        }
    }
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

                    if fd == 0 {
                        bail!("Unsupported file descriptor: {}", fd)
                    }

                    match cpu.kernel.close_fd(fd) {
                        Ok(_) => {
                            cpu.write_x_u32(ABIRegister::A(0).to_x_reg_id() as u8, 0)?;
                        }
                        Err(_e) => {
                            cpu.write_x_i32(ABIRegister::A(0).to_x_reg_id() as u8, -1)?;
                            cpu.write_x_u32(ABIRegister::A(10).to_x_reg_id() as u8, 1)?;
                        }
                    }
                }
                62 => {
                    // Seek syscall
                    let fd = cpu.read_x_u32(ABIRegister::A(0).to_x_reg_id() as u8)?;
                    let offset = cpu.read_x_u32(ABIRegister::A(1).to_x_reg_id() as u8)?;
                    let seek_type = SeekType::from(cpu.read_x_u32(ABIRegister::A(2).to_x_reg_id() as u8)?);

                    cpu.debug_print(|| format!("seek: {} {} {:?}", fd, offset, seek_type));

                    if fd == 0 {
                        bail!("Unsupported file descriptor: {}", fd)
                    }

                    let res: Result<u64> = cpu.kernel.seek_fd(fd, offset as usize, seek_type);

                    match res {
                        Ok(len) => {
                            cpu.write_x_u32(ABIRegister::A(0).to_x_reg_id() as u8, len as u32)?;
                        }
                        Err(_e) => {
                            cpu.write_x_i32(ABIRegister::A(0).to_x_reg_id() as u8, -1)?;
                            cpu.write_x_u32(ABIRegister::A(10).to_x_reg_id() as u8, 1)?;
                        }
                    }
                }
                63 => {
                    // Read syscall
                    let fd = cpu.read_x_u32(ABIRegister::A(0).to_x_reg_id() as u8)?;
                    let buffer_addr = cpu.read_x_u32(ABIRegister::A(1).to_x_reg_id() as u8)?;
                    let len = cpu.read_x_u32(ABIRegister::A(2).to_x_reg_id() as u8)?;

                    cpu.debug_print(|| format!("read: {} {} {}", fd, buffer_addr, len));

                    if fd == 0 {
                        bail!("Unsupported file descriptor: {}", fd)
                    }

                    let mut buf = vec![0; len as usize];

                    let res: Result<usize> = cpu.kernel.read_fd(fd, buf.as_mut_slice());

                    match res {
                        Ok(len) => {
                            cpu.write_buf(buffer_addr, buf.as_mut_slice())?;
                            cpu.write_x_u32(ABIRegister::A(0).to_x_reg_id() as u8, len as u32)?;
                        }
                        Err(_e) => {
                            cpu.write_x_i32(ABIRegister::A(0).to_x_reg_id() as u8, -1)?;
                            cpu.write_x_u32(ABIRegister::A(10).to_x_reg_id() as u8, 1)?;
                        }
                    }
                }
                64 => {
                    // Write syscall
                    let fd = cpu.read_x_u32(ABIRegister::A(0).to_x_reg_id() as u8)?;
                    let buffer_addr = cpu.read_x_u32(ABIRegister::A(1).to_x_reg_id() as u8)?;
                    let len = cpu.read_x_u32(ABIRegister::A(2).to_x_reg_id() as u8)?;

                    if fd == 0 {
                        bail!("Unsupported file descriptor: {}", fd)
                    }

                    let mut buf = vec![0; len as usize];
                    cpu.read_buf(buffer_addr, buf.as_mut_slice())?;

                    let res: Result<u32> = match fd {
                        1 => {
                            cpu.write_stdout(buf.as_mut_slice());
                            Ok(buf.len() as u32)
                        }
                        2 => {
                            cpu.write_stderr(buf.as_mut_slice());
                            Ok(buf.len() as u32)
                        }
                        other => Ok(cpu.kernel.write_fd(other, buf.as_mut_slice())? as u32),
                    };

                    match res {
                        Ok(len) => {
                            cpu.write_x_u32(ABIRegister::A(0).to_x_reg_id() as u8, len)?;
                        }
                        Err(_e) => {
                            cpu.write_x_i32(ABIRegister::A(0).to_x_reg_id() as u8, -1)?;
                            cpu.write_x_u32(ABIRegister::A(10).to_x_reg_id() as u8, 1)?;
                        }
                    }
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
                    let stat =if fd == 1 {
                        Stat::new_stdout(cpu.stdout_buffer.len() as u32)
                    } else {
                        Stat::from(cpu.kernel.fstat_fd(fd)?)
                    };
                    
                    cpu.write_x_u32(ABIRegister::A(0).to_x_reg_id() as u8, 0)?;
                    cpu.write_buf(stat_addr, &stat.to_bytes() as &[u8])?;
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

                    let seconds = now.tv_sec() as u64;
                    let nanos = now.tv_nsec() as u32;

                    cpu.write_mem_u32(timespec_addr, (seconds >> 32) as u32)?;
                    cpu.write_mem_u32(timespec_addr + 4, (seconds) as u32)?;
                    cpu.write_mem_u32(timespec_addr + 8, nanos as u32)?;

                    cpu.write_x_u32(ABIRegister::A(0).to_x_reg_id() as u8, 0)?;

                    // cpu.debug_print(|| format!("clock_gettime: {} {}", seconds, nanos));
                    println!("clock_gettime: {} {}", seconds, nanos);
                }
                1024 => {
                    // open
                    let path_addr = cpu.read_x_u32(ABIRegister::A(0).to_x_reg_id() as u8)?;
                    let path = cpu.read_c_string(path_addr)?;
                    let flags = cpu.read_x_u32(ABIRegister::A(1).to_x_reg_id() as u8)?;

                    println!("open file: {} {:#x}", path, flags);

                    match cpu.kernel.open_file(&path) {
                        Ok(fd) => {
                            cpu.write_x_u32(ABIRegister::A(0).to_x_reg_id() as u8, fd)?;
                            println!("Opened file: {} with fd {}", path, fd);
                        }
                        Err(_e) => {
                            cpu.write_x_i32(ABIRegister::A(0).to_x_reg_id() as u8, -1)?;
                            cpu.write_x_u32(ABIRegister::A(1).to_x_reg_id() as u8, 1)?;
                            println!("Failed to open file: {}", path);
                        }
                    }
                }
                code => {
                    println!("Unsupported syscall: {}", code)
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
