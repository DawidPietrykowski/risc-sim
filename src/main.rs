#![feature(let_chains)]

use anyhow::Result;
use clap::Parser;
use cli_utils::{print_debug_info, setup_cpu, setup_terminal, CliArgs};
use ctrlc::set_handler;
use doom::{doom_init, update_window, DoomEmulation};
use risc_sim::cpu::cpu_core::ExecutionMode;
use risc_sim::system::uart::write_char;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod cli_utils;
mod doom;

fn main() -> Result<()> {
    let args = CliArgs::parse();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let stdio_channel = setup_terminal()?;

    let mut cpu = setup_cpu(&args)?;

    let mut emulation: Option<DoomEmulation> = if args.simulate_display {
        Some(doom_init())
    } else {
        None
    };

    let start_time = std::time::Instant::now();

    let mut count = 0;
    const COUNT_INTERVAL: u64 = 5000;
    let mut stdio_count = 0;
    const STDIO_READ_INTERVAL: u64 = 500000;
    let res = loop {
        if !running.load(Ordering::SeqCst) {
            break anyhow::anyhow!("Interrupted by Ctrl-C");
        }

        count += COUNT_INTERVAL;

        if args.execution_mode == ExecutionMode::Bare {
            if stdio_count > STDIO_READ_INTERVAL {
                if let Ok(c) = stdio_channel.try_recv() {
                    if c == 3 {
                        break anyhow::anyhow!("Interrupted by Ctrl-C");
                    } else {
                        write_char(&mut cpu, c);
                    }
                }
                stdio_count = 0;
            } else {
                stdio_count += COUNT_INTERVAL;
            }
        }

        if args.simulate_display {
            if let Err(e) = update_window(&mut cpu, emulation.as_mut().unwrap()) {
                break e;
            }
        }

        if let Some(timeout) = args.timeout
            && start_time.elapsed().as_secs_f32() >= timeout as f32
        {
            break anyhow::anyhow!("Timeout");
        }

        match cpu.run_cycles(COUNT_INTERVAL) {
            Ok(_) => {
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
    if let Some(emulation) = emulation {
        println!(
            "FPS: {}",
            emulation.frames_drawn as f64 / elapsed_time.as_secs_f64()
        );
    }
    print_debug_info(cpu, count, elapsed_time);

    Ok(())
}
