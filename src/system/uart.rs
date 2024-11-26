use crate::cpu::cpu_core::Cpu;

const UART_ADDR: u64 = 0x10000000;
const LSR_REG: u64 = 0x5;
const LSR_RX_READY: u8 = 1 << 0;
#[allow(dead_code)]
const LSR_TX_READY: u8 = 1 << 5;

pub fn read_uart_pending(cpu: &mut Cpu) -> Option<u8> {
    let pending = cpu.read_mem_u8(UART_ADDR + LSR_REG).unwrap() & LSR_RX_READY;
    if pending == 1 {
        Some(cpu.read_mem_u8(UART_ADDR).unwrap())
    } else {
        None
    }
}
