use crate::cpu::cpu_core::Cpu;

pub const UART_ADDR: u64 = 0x10000000;
const LSR_REG: u64 = 0x5;
#[allow(dead_code)]
const LSR_RX_READY: u8 = 1 << 0;
const LSR_TX_READY: u8 = 1 << 5;

pub fn read_uart_pending(cpu: &mut Cpu) -> Option<u8> {
    let lsr = cpu.read_mem_u8(UART_ADDR + LSR_REG).unwrap();
    cpu.write_mem_u8(UART_ADDR + LSR_REG, lsr | LSR_TX_READY)
        .unwrap();
    Some(cpu.read_mem_u8(UART_ADDR).unwrap())
}

pub fn init_uart(cpu: &mut Cpu) {
    cpu.write_mem_u8(UART_ADDR + LSR_REG, LSR_TX_READY).unwrap();
}
