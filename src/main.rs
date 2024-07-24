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
    // let mut memory: Memory = Memory::new();

    // let memory_addr = 0x4;
    // let val = 0xFF00F000;
    // memory.write_mem_u32(memory_addr, val);
    // // memory.write_mem_u8(memory_addr + 0, 0x0F);
    // // memory.write_mem_u8(memory_addr + 1, 0x0F);
    // // memory.write_mem_u8(memory_addr + 2, 0x0F);
    // // memory.write_mem_u8(memory_addr + 3, 0x0F);
    // // memory.write_mem_u16(memory_addr + 3, 0xabcd);

    // let read_val = memory.read_mem_u32(memory_addr + 0)?;
    // println!("Read value: {:#x}", read_val);
    // // assert_eq!(read_val, val);

    // // memory.write_mem_u8(memory_addr + 4, 0xaa);

    // // let read_val = memory.read_mem_u32(memory_addr + 4)?;
    // // println!("Read value: {:#x}", read_val);

    // let read_val = memory.read_mem_u16(memory_addr + 1)?;
    // println!("Read value u16: {:#x}", read_val);
    // return Ok(());

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(anyhow::anyhow!("Usage: {} <path_to_file>", args[0]));
    }
    let file_path = &args[1];
    let program = decode_file(file_path);

    let mut cpu = Cpu::new();
    cpu.load_program(program.memory, program.entry_point);

    let start_time = std::time::Instant::now();

    let mut count = 0;
    let res = loop {
        count += 1;
        if count > MAX_CYCLES {
            return Err(anyhow::anyhow!("Too many cycles"));
        }
        // println!("Cycles: {}", count);
        match cpu.run_cycle() {
            Ok(_) => continue,
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
