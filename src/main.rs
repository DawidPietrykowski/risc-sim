#![feature(let_chains)]
#![allow(dead_code)]

use anyhow::Result;
use clap::Parser;
use ctrlc::set_handler;
use minifb::{Key, Window, WindowOptions};
use nix::libc::{BRKINT, ECHO, ICRNL, INPCK, ISTRIP};
use risc_sim::cpu::cpu_core::{
    Cpu, CpuMode, ExecutionMode, INITIAL_STACK_POINTER_32, INITIAL_STACK_POINTER_64,
};
use risc_sim::cpu::memory::raw_memory::ContinuousMemory;
use risc_sim::cpu::memory::user_memory::{UserMemory, HEAP_SIZE, STACK_SIZE};
use risc_sim::elf::elf_loader::{decode_file, WordSize};
use risc_sim::isa::csr::csr_types::CSRAddress;
use risc_sim::system::passthrough_kernel::PassthroughKernel;
use risc_sim::system::uart::{init_uart, write_char};
use risc_sim::system::virtio::{init_virtio, BlockDevice};
use risc_sim::types::ABIRegister;
use std::collections::HashMap;
use std::io::{self, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread;
use termios::{Termios, ICANON, IXON, TCSANOW, VMIN, VTIME};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the program to execute
    #[arg(required = true)]
    pub program_path: String,

    /// Enable display simulation
    #[arg(long, default_value_t = false)]
    pub simulate_display: bool,

    /// Execution mode (userspace/bare)
    #[arg(long, value_enum, default_value_t = ExecutionMode::UserSpace)]
    pub execution_mode: ExecutionMode,

    /// Optional filesystem image path
    #[arg(long)]
    pub fs_image: Option<String>,

    /// Optional timeout
    #[arg(long)]
    pub timeout: Option<u32>,
}

fn create_stdio_channel() -> Receiver<u8> {
    let (tx, rx) = mpsc::channel::<u8>();
    thread::spawn(move || loop {
        let mut buf = [0u8; 1];
        let n = io::stdin().read(&mut buf).unwrap();
        if n != 0 {
            let c = buf[0];
            tx.send(c).unwrap();
        }
    });
    rx
}

pub const KEY_RIGHTARROW: u8 = 0xae;
pub const KEY_LEFTARROW: u8 = 0xac;
pub const KEY_UPARROW: u8 = 0xad;
pub const KEY_DOWNARROW: u8 = 0xaf;
pub const KEY_STRAFE_L: u8 = 0xa0;
pub const KEY_STRAFE_R: u8 = 0xa1;
pub const KEY_USE: u8 = 0xa2;
pub const KEY_FIRE: u8 = 0xa3;
pub const KEY_ESCAPE: u8 = 27;
pub const KEY_ENTER: u8 = 13;
pub const KEY_TAB: u8 = 9;

fn main() -> Result<()> {
    let args = Args::parse();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    const SCREEN_WIDTH: u64 = 320;
    const SCREEN_HEIGHT: u64 = 200;
    const MEMORY_BUFFER_SIZE: u64 = SCREEN_WIDTH * SCREEN_HEIGHT * 4;
    const SCREEN_ADDR_ADDR: u64 = 0x1000000 - 4;
    const KEYQUEUE_ADDR_ADDR: u64 = 0x1000000 - 8;
    const SCALE_SCREEN: u64 = 6;
    let simulate_display: bool = args.simulate_display;

    let key_pairs = vec![
        (Key::W, KEY_UPARROW),
        (Key::S, KEY_DOWNARROW),
        (Key::Left, KEY_LEFTARROW),
        (Key::Right, KEY_RIGHTARROW),
        (Key::Enter, KEY_ENTER),
        (Key::Tab, KEY_TAB),
        (Key::E, KEY_FIRE),
        (Key::Q, KEY_USE),
        (Key::A, KEY_STRAFE_L),
        (Key::D, KEY_STRAFE_R),
        (Key::Escape, KEY_ESCAPE),
    ];
    let mut key_states: HashMap<Key, bool> = HashMap::new();
    for (key, _) in &key_pairs {
        key_states.insert(*key, false);
    }
    let mut frames_written = 0;

    let mut window = if simulate_display {
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

    let mut buffer: Vec<u32> = if simulate_display {
        let buf: Vec<u32> = vec![0; (SCREEN_WIDTH * SCREEN_HEIGHT).try_into().unwrap()];

        window
            .as_mut()
            .unwrap()
            .update_with_buffer(&buf, SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize)
            .unwrap();
        buf
    } else {
        Vec::new()
    };

    let mut termios = Termios::from_fd(0)?;
    // Disable canonical mode, echo, and other processing
    termios.c_lflag &= !(ICANON | ECHO);
    // Disable various input processing
    termios.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
    // Set read timeout and minimum input
    termios.c_cc[VMIN] = 0; // Minimum number of characters
    termios.c_cc[VTIME] = 0; // Timeout in deciseconds
    termios::tcsetattr(0, TCSANOW, &termios)?;
    let stdio_channel = create_stdio_channel();

    let program = decode_file(&args.program_path);

    let mode = if program.header.word_size == WordSize::W32 {
        CpuMode::RV32
    } else {
        CpuMode::RV64
    };
    let block_dev = if let Some(path) = args.fs_image {
        Some(BlockDevice::new(&path)?)
    } else {
        None
    };
    let mut cpu = match args.execution_mode {
        ExecutionMode::Bare => Cpu::new(
            ContinuousMemory::default(),
            PassthroughKernel::default(),
            mode,
            block_dev,
            args.execution_mode.clone(),
        ),
        ExecutionMode::UserSpace => match mode {
            CpuMode::RV64 => Cpu::new(
                UserMemory::new(
                    INITIAL_STACK_POINTER_64 as u64 - STACK_SIZE,
                    0,
                    STACK_SIZE,
                    HEAP_SIZE,
                ),
                PassthroughKernel::default(),
                mode,
                block_dev,
                args.execution_mode.clone(),
            ),
            CpuMode::RV32 => Cpu::new(
                UserMemory::new(
                    INITIAL_STACK_POINTER_32 as u64 - STACK_SIZE,
                    0,
                    STACK_SIZE,
                    HEAP_SIZE,
                ),
                PassthroughKernel::default(),
                mode,
                block_dev,
                args.execution_mode.clone(),
            ),
        },
    };
    cpu.load_program_from_elf(program)?;

    init_uart(&mut cpu);
    init_virtio(&mut cpu);

    let start_time = std::time::Instant::now();

    let mut count = 0;
    #[cfg(feature = "maxperf")]
    const COUNT_INTERVAL: u64 = 5000;
    #[cfg(not(feature = "maxperf"))]
    const COUNT_INTERVAL: u64 = 1;
    let mut stdio_count = 0;
    const STDIO_READ_INTERVAL: u64 = 2;
    let res = loop {
        if !running.load(Ordering::SeqCst) {
            break anyhow::anyhow!("Interrupted by Ctrl-C");
        }

        count += COUNT_INTERVAL;

        if args.execution_mode == ExecutionMode::Bare {
            if stdio_count % STDIO_READ_INTERVAL == 0 {
                if let Ok(c) = stdio_channel.try_recv() {
                    if c == 3 {
                        break anyhow::anyhow!("Interrupted by Ctrl-C");
                    } else {
                        write_char(&mut cpu, c);
                    }
                }
                stdio_count += 1;
            } else {
                stdio_count += 1;
            }
        }

        if simulate_display && cpu.read_mem_u32(SCREEN_ADDR_ADDR)? != 0 {
            frames_written += 1;

            let window = window.as_mut().unwrap();
            if window.is_key_down(Key::Backslash) {
                break anyhow::anyhow!("Escape pressed");
            }
            println!("Draw on cycle: {}", count);

            let screen_data_addr = cpu.read_mem_u32(SCREEN_ADDR_ADDR)? as u64;

            cpu.write_mem_u32(SCREEN_ADDR_ADDR, 0)?;

            let cmap = false;

            for ypos in 0..SCREEN_HEIGHT {
                for xpos in 0..SCREEN_WIDTH {
                    let pixel_index = (xpos) * SCREEN_HEIGHT + (ypos);
                    let g;
                    let b;
                    let r;
                    if cmap {
                        let val = cpu.read_mem_u8(screen_data_addr + pixel_index)? as u32;
                        g = ((val) & 0b11) * 0xFF / 4;
                        b = ((val >> 3) & 0b11) * 0xFF / 4;
                        r = ((val >> 6) & 0b11) * 0xFF / 4;
                    } else {
                        let val = cpu.read_mem_u32(screen_data_addr + pixel_index * 4)?;
                        r = (val) & 0xFF;
                        g = (val >> 8) & 0xFF;
                        b = (val >> 16) & 0xFF;
                    }
                    buffer[pixel_index as usize] = r | (g << 8) | (b << 16) | (0xFF << 24);
                }
            }

            if window.is_open() {
                window
                    .update_with_buffer(&buffer, SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize)
                    .unwrap();
            }

            let keyqueue_data_addr = cpu.read_mem_u32(KEYQUEUE_ADDR_ADDR)? as u64;
            let keyqueue_data = cpu.read_mem_u32(keyqueue_data_addr)?;
            if keyqueue_data != 0xFFFF_FFFF {
                continue;
            }
            let mut queue_entry_count = 0;
            for (key, doom_key) in &key_pairs {
                let down = window.is_key_down(*key);
                let down_prev = key_states[&key];
                let pressed = down && !down_prev;
                let released = !down && down_prev;
                let state = key_states.get_mut(key).unwrap();
                *state = down;
                if pressed || released {
                    cpu.write_mem_u32(
                        keyqueue_data_addr + queue_entry_count * 4,
                        ((pressed as u32) << 31) | *doom_key as u32,
                    )
                    .unwrap();
                    queue_entry_count += 1;
                }
            }
            cpu.write_mem_u32(keyqueue_data_addr + queue_entry_count * 4, 0xFFFFFFFF)
                .unwrap();
        }

        if let Some(timeout) = args.timeout
            && start_time.elapsed().as_secs_f32() >= timeout as f32
        {
            break anyhow::anyhow!("Timeout");
        }

        #[cfg(not(feature = "maxperf"))]
        match cpu.run_cycle() {
            Ok(_) => {
                continue;
            }
            Err(e) => {
                break e;
            }
        }
        #[cfg(feature = "maxperf")]
        let _ = cpu.run_cycles(COUNT_INTERVAL);
    };

    let elapsed_time = start_time.elapsed();

    println!();
    println!("Execution stopped due to: {:?}", res);
    println!("CPU state: \n{}", cpu);

    cpu.print_pc_history();

    let exit_code = cpu.read_x_u32(ABIRegister::A(0).to_x_reg_id() as u8);
    println!("Program exit code: {}", exit_code);
    if count > 1_000_000 {
        println!("Total cycle count: {} k", count / 1_000);
    } else {
        println!("Total cycle count: {}", count);
    }
    println!("Elapsed time: {:?}", elapsed_time);
    println!(
        "FPS: {}",
        frames_written as f64 / elapsed_time.as_secs_f64()
    );
    println!(
        "Cycles per second: {} mln",
        (count as f64 / elapsed_time.as_secs_f64()) as u64 / 1_000_000
    );
    println!(
        "SATP: {:x}",
        cpu.csr_table.read64(CSRAddress::Satp.as_u12())
    );
    println!(
        "Translated 0x0: {:x}",
        cpu.translate_address_if_needed(0x0).unwrap()
    );
    println!(
        "Translated PC: {:x}",
        cpu.translate_address_if_needed(cpu.read_pc_u64()).unwrap()
    );
    println!(
        "WORD at PC: {:x}",
        cpu.read_mem_u32(cpu.read_pc_u64()).unwrap()
    );

    Ok(())
}
