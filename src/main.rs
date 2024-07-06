#![allow(dead_code)]

use anyhow::{Ok, Result};
use asm::assembler::decode_file;
use isa::cpu::Cpu;

mod asm;
mod isa;
mod test;
mod utils;

fn main() -> Result<()> {
    let program = decode_file();

    let mut cpu = Cpu::new();
    cpu.load_program(program);

    while cpu.run_cycle().is_ok() {}

    let result = cpu.read_mem_u32(0x00001000).unwrap();

    println!("Result: {}", result);

    Ok(())
}
