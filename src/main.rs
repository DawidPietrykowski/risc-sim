#![allow(dead_code)]

use anyhow::Result;
use minifb::{Key, Window, WindowOptions};
use risc_sim::cpu::cpu_core::Cpu;
use risc_sim::elf::elf_loader::decode_file;
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
    const MEMORY_BUFFER_SIZE: u32 = SCREEN_WIDTH * SCREEN_HEIGHT * 4;
    const SCREEN_ADDR: u32 = 0x40000000;
    const SCALE_SCREEN: u32 = 2;
    const SIMULATE_DISPLAY: bool = false;

    let mut frames_written = 0;

    let mut window = if SIMULATE_DISPLAY {
        Some(
            Window::new(
                "DISPLAY",
                (SCALE_SCREEN * SCREEN_WIDTH) as usize,
                (SCALE_SCREEN * SCREEN_HEIGHT) as usize,
                WindowOptions::default(),
            )
            .unwrap_or_else(|e| {
                panic!("{}", e);
            }),
        )
    } else {
        None
    };

    let mut buffer: Vec<u32> = if SIMULATE_DISPLAY {
        let buf: Vec<u32> = vec![
            0;
            (SCREEN_WIDTH * SCREEN_HEIGHT * SCALE_SCREEN * SCALE_SCREEN)
                .try_into()
                .unwrap()
        ];

        window
            .as_mut()
            .unwrap()
            .update_with_buffer(&buf, SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize)
            .unwrap();
        buf
    } else {
        Vec::new()
    };

    // delay for 10s
    // std::thread::sleep(std::time::Duration::from_secs(10));

    let file_path = &args[1];
    let program = decode_file(file_path);

    let mut cpu = Cpu::default();
    cpu.load_program_from_elf(program)?;
    // cpu.set_debug_enabled(true);

    let start_time = std::time::Instant::now();

    let mut count = 0;
    const COUNT_INTERVAL: u64 = 5000000;
    let res = loop {
        #[cfg(not(feature = "maxperf"))]
        {
            count += 1;
        }
        #[cfg(feature = "maxperf")]
        {
            count += COUNT_INTERVAL;
        }

        if count > MAX_CYCLES {
            break anyhow::anyhow!("Too many cycles");
        }

        if cpu.read_mem_u8(SCREEN_ADDR + MEMORY_BUFFER_SIZE)? != 0 {
            frames_written += 1;

            println!("Draw on cycle: {}", count);

            cpu.write_mem_u8(SCREEN_ADDR + MEMORY_BUFFER_SIZE, 0)?;
        }
        if SIMULATE_DISPLAY && cpu.read_mem_u8(SCREEN_ADDR + MEMORY_BUFFER_SIZE)? != 0 {
            frames_written += 1;

            println!("Draw on cycle: {}", count);

            cpu.write_mem_u8(SCREEN_ADDR + MEMORY_BUFFER_SIZE, 0)?;

            if window.as_ref().unwrap().is_key_down(Key::Escape) {
                break anyhow::anyhow!("Escape pressed");
            }

            let cmap = false;

            for ypos in 0..SCREEN_HEIGHT {
                for xpos in 0..SCREEN_WIDTH {
                    let pixel_index = (xpos) * SCREEN_HEIGHT + (ypos);
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
                    for xt in 0..SCALE_SCREEN {
                        for yt in 0..SCALE_SCREEN {
                            let pixel_index = (xpos + xt) * (SCREEN_HEIGHT) + (ypos + yt);
                            buffer[pixel_index as usize] = r | (g << 8) | (b << 16) | (0xFF << 24);
                        }
                    }
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
        #[cfg(not(feature = "maxperf"))]
        match cpu.run_cycle() {
            Ok(_) => {
                // println!("Cycle: {}", count);
                continue;
            }
            Err(e) => {
                break e;
            }
        }
        #[cfg(feature = "maxperf")]
        {
            let mut finished = false;
            for _ in 0..COUNT_INTERVAL {
                match cpu.run_cycle_uncheked() {
                    Ok(_) => {
                        // println!("Cycle: {}", count);
                        continue;
                    }
                    Err(_e) => {
                        finished = true;
                        break;
                    }
                };
            }
            if finished {
                break anyhow::anyhow!("Error");
            }
            if start_time.elapsed() > std::time::Duration::from_secs(30) {
                break anyhow::anyhow!("Timeout");
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
        "FPS: {}",
        frames_written as f64 / elapsed_time.as_secs_f64()
    );
    println!(
        "Cycles per second: {} mln",
        (count as f64 / elapsed_time.as_secs_f64()) as u64 / 1_000_000
    );
    // println!("\nSTDOUT buffer:\n{}", cpu.read_and_clear_stdout_buffer());

    Ok(())
}
