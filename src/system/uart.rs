use crate::cpu::{cpu_core::Cpu, memory::memory_core::Memory};

use super::plic::plic_trigger_irq;
use std::io::{self, Write};

pub const UART_ADDR: u64 = 0x10000000;

const LSR_REG: u64 = 0x5;

const LSR_RX_READY: u8 = 1 << 0;
const LSR_TX_READY: u8 = 1 << 5;

const UART0_IRQ: u32 = 10;

pub fn uart_handle_write(cpu: &mut Cpu, value: u8) {
    let uart = &mut cpu.peripherals.as_mut().unwrap().uart;
    let mut stdout = io::stdout();
    let lsr = uart.read_mem_u8(UART_ADDR + LSR_REG).unwrap();
    uart.write_mem_u8(UART_ADDR + LSR_REG, lsr | LSR_TX_READY)
        .unwrap();
    uart.write_mem_u8(UART_ADDR, value).unwrap();
    write!(stdout, "\x1b[93m{}\x1b[0m", value as char).unwrap();

    stdout.flush().unwrap();
}

pub fn init_uart(cpu: &mut Cpu) {
    let uart = &mut cpu.peripherals.as_mut().unwrap().uart;
    uart.write_mem_u8(UART_ADDR + LSR_REG, LSR_TX_READY)
        .unwrap();
}

pub fn write_char(cpu: &mut Cpu, c: u8) {
    let uart = &mut cpu.peripherals.as_mut().unwrap().uart;
    let lsr = uart.read_mem_u8(UART_ADDR + LSR_REG).unwrap();
    if lsr & LSR_RX_READY == 0 {
        uart.write_mem_u8(UART_ADDR + LSR_REG, lsr | LSR_RX_READY)
            .unwrap();

        uart.write_mem_u8(UART_ADDR, c).unwrap();

        plic_trigger_irq(cpu, UART0_IRQ);
    }
}

pub fn uart_handle_read(cpu: &mut Cpu) -> u8 {
    let uart = &mut cpu.peripherals.as_mut().unwrap().uart;
    let lsr = uart.read_mem_u8(UART_ADDR + LSR_REG).unwrap();

    uart.write_mem_u8(UART_ADDR + LSR_REG, lsr & !LSR_RX_READY)
        .unwrap();
    uart.read_mem_u8(UART_ADDR).unwrap()
}
