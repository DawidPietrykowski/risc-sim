#![allow(dead_code)]

use anyhow::Result;
use asm::assembler::decode_file;
use isa::cpu::Cpu;

mod asm;
mod isa;
mod test;
mod utils;

fn main() -> Result<()> {
    let program = decode_file("notes/simple");

    let mut cpu = Cpu::new();
    cpu.load_program(program.memory, program.entry_point);

    let mut count = 0;
    loop {
        count += 1;
        if count > 10000 {
            return Err(anyhow::anyhow!("Too many cycles"));
        }
        match cpu.run_cycle() {
            Ok(_) => continue,
            Err(e) => return Err(e.into()),
        }
    };
    // println!("{:?}", res);

    let result = cpu.read_mem_u32(0x00001000).unwrap();

    println!("Result: {}", result);

    Ok(())
}
