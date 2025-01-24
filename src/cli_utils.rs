use anyhow::Result;
use clap::Parser;
use nix::libc::{BRKINT, ECHO, ICRNL, INPCK, ISTRIP};
use risc_sim::cpu::cpu_core::{Cpu, CpuMode, ExecutionMode};
use risc_sim::elf::elf_loader::{decode_file, WordSize};
use risc_sim::isa::csr::csr_types::CSRAddress;
use risc_sim::system::uart::init_uart;
use risc_sim::system::virtio::{init_virtio, BlockDevice};
use risc_sim::types::ABIRegister;
use risc_sim::utils::data::print_pc_history;
use std::io::Read;
use std::sync::mpsc::{self, Receiver};
use std::{io, thread};
use termios::{Termios, ICANON, IXON, TCSANOW, VMIN, VTIME};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
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

pub fn print_debug_info(mut cpu: Cpu, count: u64, elapsed_time: std::time::Duration) {
    println!("CPU state: \n{}", cpu);

    print_pc_history(&mut cpu.pc_history);

    let exit_code = cpu.read_x_u32(ABIRegister::A(0).to_x_reg_id() as u8);
    println!("Program exit code: {}", exit_code);
    if count > 1_000_000 {
        println!("Total cycle count: {} k", count / 1_000);
    } else {
        println!("Total cycle count: {}", count);
    }
    println!("Elapsed time: {:?}", elapsed_time);
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
}

pub fn setup_cpu(args: &CliArgs) -> Result<Cpu> {
    let program = decode_file(&args.program_path);
    let mode = if program.header.word_size == WordSize::W32 {
        CpuMode::RV32
    } else {
        CpuMode::RV64
    };
    let block_dev = if let Some(path) = &args.fs_image {
        Some(BlockDevice::new(&path)?)
    } else {
        None
    };
    let mut cpu = match args.execution_mode {
        ExecutionMode::Bare => Cpu::new_bare(block_dev),
        ExecutionMode::UserSpace => Cpu::new_userspace(mode),
    };
    cpu.load_program_from_elf(program)?;
    init_uart(&mut cpu);
    init_virtio(&mut cpu);
    Ok(cpu)
}

pub fn setup_terminal() -> Result<Receiver<u8>> {
    let mut termios = Termios::from_fd(0)?;
    termios.c_lflag &= !(ICANON | ECHO);
    termios.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
    termios.c_cc[VMIN] = 0;
    termios.c_cc[VTIME] = 0;
    termios::tcsetattr(0, TCSANOW, &termios)?;
    let stdio_channel = create_stdio_channel();
    Ok(stdio_channel)
}

pub fn create_stdio_channel() -> Receiver<u8> {
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
