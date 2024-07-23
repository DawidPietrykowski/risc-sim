#![allow(dead_code)]

use anyhow::Result;
use asm::assembler::decode_file;
use isa::{cpu::Cpu, types::ABIRegister};

mod asm;
mod isa;
mod test;
mod utils;

fn main() -> Result<()> {
    let program = decode_file("notes/printf");

    let mut cpu = Cpu::new();
    cpu.load_program(program.memory, program.entry_point);

    // return Ok(());
    let mut count = 0;
    let res = loop {
        count += 1;
        if count > 10000 {
            return Err(anyhow::anyhow!("Too many cycles"));
        }
        println!("Cycles: {}", count);
        match cpu.run_cycle() {
            Ok(_) => continue,
            Err(e) => {
                break e;
            },
        }
    };

    println!("{:?}", res);
    let exit_code = cpu.read_x_u32(ABIRegister::A(0).to_x_reg_id() as u8)?;
    println!("Exit code: {}", exit_code);

    let result = cpu.read_mem_u32(0x011101ac).unwrap();

    println!("Result: {}", result);

    Ok(())
}
