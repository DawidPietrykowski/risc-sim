#![allow(dead_code)]

use anyhow::Result;
use asm::assembler::decode_file;
use isa::{cpu::Cpu, memory::Memory, types::ABIRegister};

mod asm;
mod isa;
mod test;
mod utils;

use std::env;

const MAX_CYCLES: u32 = 100000000;

fn main() -> Result<()> {
    if false {
        let mut memory: Memory = Memory::new();

        memory.write_mem_u32(0x21054, 0x10c78)?;
        let read_val = memory.read_mem_u32(0x21054)?;
        println!("Read value: {:#010X}", read_val);
        let read_val = memory.read_mem_u8(0x21056)?;
        println!("Read value: {:#010X}", read_val);
        return Ok(());

        let addr = 4096 - 3;

        let value = 0x12345678u32;
        // let value = 0xFF00F001;
        memory.write_mem_u32(addr, value).unwrap();

        println!("{:#010X}", value);
        println!("{:#010X}", memory.read_mem_u32(0).unwrap());
        println!("{:#010X}", memory.read_mem_u32(4).unwrap());

        assert_eq!(memory.read_mem_u32(addr).unwrap(), value);

        assert_eq!(memory.read_mem_u8(addr).unwrap(), 0x78);
        assert_eq!(memory.read_mem_u8(addr + 1).unwrap(), 0x56);
        assert_eq!(memory.read_mem_u8(addr + 2).unwrap(), 0x34);
        assert_eq!(memory.read_mem_u8(addr + 3).unwrap(), 0x12);
        // assert_eq!(memory.read_mem_u8(addr).unwrap(), 0x01);
        // assert_eq!(memory.read_mem_u8(addr + 1).unwrap(), 0xF0);
        // assert_eq!(memory.read_mem_u8(addr + 2).unwrap(), 0x00);
        // assert_eq!(memory.read_mem_u8(addr + 3).unwrap(), 0xFF);

        // assert_eq!(memory.read_mem_u16(addr).unwrap(), 0xF001);
        // assert_eq!(memory.read_mem_u16(addr + 2).unwrap(), 0xFF00);

        assert_eq!(memory.read_mem_u16(addr).unwrap(), 0x5678);
        assert_eq!(memory.read_mem_u16(addr + 1).unwrap(), 0x3456);
        assert_eq!(memory.read_mem_u16(addr + 2).unwrap(), 0x1234);
        assert_eq!(memory.read_mem_u16(addr + 3).unwrap(), 0x0012);

        return Ok(());

        let memory_addr = 0x0;
        let val = 0xFF00F000;
        memory.write_mem_u32(memory_addr, val);
        // memory.write_mem_u8(memory_addr + 0, 0x0F);
        // memory.write_mem_u8(memory_addr + 1, 0x0F);
        // memory.write_mem_u8(memory_addr + 2, 0x0F);
        // memory.write_mem_u8(memory_addr + 3, 0x0F);
        // memory.write_mem_u16(memory_addr + 3, 0xabcd);

        let read_val = memory.read_mem_u32(memory_addr + 0)?;
        println!("Read value: {:#x}", read_val);
        // assert_eq!(read_val, val);

        // memory.write_mem_u8(memory_addr + 4, 0xaa);

        // let read_val = memory.read_mem_u32(memory_addr + 4)?;
        // println!("Read value: {:#x}", read_val);

        let read_val = memory.read_mem_u16(memory_addr + 0)?;
        println!("Read value u16: {:#x}", read_val);
        return Ok(());
    }
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(anyhow::anyhow!("Usage: {} <path_to_file>", args[0]));
    }
    let file_path = &args[1];
    let program = decode_file(file_path);

    let mut cpu = Cpu::new();
    cpu.load_program(program.memory, program.entry_point);
    // cpu.set_debug_enabled(true);

    let start_time = std::time::Instant::now();

    let mut count = 0;
    let res = loop {
        count += 1;
        if count > MAX_CYCLES {
            return Err(anyhow::anyhow!("Too many cycles"));
        }
        match cpu.run_cycle() {
            Ok(_) => {
                // println!("Cycle: {}", count);
                continue;
            }
            Err(e) => {
                break e;
            }
        }
    };

    let elapsed_time = start_time.elapsed();

    println!();
    println!("Execution stopped due to: {:?}", res);

    println!("CPU state: \n{}", cpu);

    let exit_code = cpu.read_x_u32(ABIRegister::A(0).to_x_reg_id() as u8)?;
    println!("Program exit code: {}", exit_code);
    println!("Total cycle count: {}", count);
    println!("Elapsed time: {:?}", elapsed_time);
    println!(
        "Cycles per second: {} k",
        (count as f64 / elapsed_time.as_secs_f64()) as u64 / 1000
    );
    println!("\nSTDOUT buffer:\n{}", cpu.read_and_clear_stdout_buffer());

    Ok(())
}
