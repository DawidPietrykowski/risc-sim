#![allow(dead_code)]

use anyhow::Result;
use asm::assembler::decode_file;
use isa::{cpu::Cpu, types::ABIRegister};

mod asm;
mod isa;
mod test;
mod utils;

use std::env;

const MAX_CYCLES: u32 = 100000000;

fn main() -> Result<()> {
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
    println!("Total cycle count: {} k", count / 1_000);
    println!("Elapsed time: {:?}", elapsed_time);
    println!(
        "Cycles per second: {} mln",
        (count as f64 / elapsed_time.as_secs_f64()) as u64 / 1_000_000
    );
    println!("\nSTDOUT buffer:\n{}", cpu.read_and_clear_stdout_buffer());

    Ok(())
}
