#![allow(dead_code)]

use anyhow::Result;
use minifb::{Key, Window, WindowOptions};
use risc_sim::asm::assembler::decode_file;
use risc_sim::cpu::cpu_core::Cpu;
use risc_sim::types::ABIRegister;

use std::env;

const MAX_CYCLES: u64 = 10000000000;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(anyhow::anyhow!("Usage: {} <path_to_file>", args[0]));
    }

    const SCREEN_WIDTH: u32 = 320;
    const SCREEN_HEIGHT: u32 = 200;
    const SCREEN_ADDR: u32 = 0x40000000;
    const SIMULATE_DISPLAY: bool = true;

    let mut window = if SIMULATE_DISPLAY {
        Some(
            Window::new(
                "DISPLAY",
                SCREEN_WIDTH as usize,
                SCREEN_HEIGHT as usize,
                WindowOptions::default(),
            )
            .unwrap_or_else(|e| {
                panic!("{}", e);
            }),
        )
    } else {
        None
    };

    let mut buffer: Vec<u32> = vec![0; (SCREEN_WIDTH * SCREEN_HEIGHT).try_into().unwrap()];

    let file_path = &args[1];
    let program = decode_file(file_path);

    let mut cpu = Cpu::new();
    cpu.load_program(program);
    // cpu.set_debug_enabled(true);

    let start_time = std::time::Instant::now();

    let mut count = 0;
    let res = loop {
        count += 1;
        if count > MAX_CYCLES {
            break anyhow::anyhow!("Too many cycles");
        }
        if SIMULATE_DISPLAY && count % 5000000 == 0 {
            println!("Draw on cycle: {}", count);

            if window.as_ref().unwrap().is_key_down(Key::Escape) {
                break anyhow::anyhow!("Escape pressed");
            }

            let cmap = false;

            for i in 0..SCREEN_HEIGHT {
                for j in 0..SCREEN_WIDTH {
                    let pixel_index = (j) * SCREEN_HEIGHT + (i);
                    let g;
                    let b;
                    let r;
                    if cmap {
                        let val = cpu.read_mem_u8(SCREEN_ADDR + pixel_index)? as u32;
                        g = ((val) & 0b11) * 0xFF / 4;
                        b = ((val >> 3) & 0b11) * 0xFF / 4;
                        r = ((val >> 6) & 0b11) * 0xFF / 4;
                    } else {
                        let val = cpu.read_mem_u32(SCREEN_ADDR + pixel_index * 4)?;
                        r = (val) & 0xFF;
                        g = (val >> 8) & 0xFF;
                        b = (val >> 16) & 0xFF;
                    }
                    buffer[pixel_index as usize] = r | (g << 8) | (b << 16) | (0xFF << 24);
                }
            }

            if window.as_ref().unwrap().is_open() {
                window
                    .as_mut()
                    .unwrap()
                    .update_with_buffer(&buffer, SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize)
                    .unwrap();
            }
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
    // println!("\nSTDOUT buffer:\n{}", cpu.read_and_clear_stdout_buffer());

    Ok(())
}
