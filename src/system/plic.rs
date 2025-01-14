use crate::{
    cpu::{cpu_core::Cpu, memory::memory_core::Memory},
    isa::{csr::csr_types::CSRAddress, traps::TrapInterruptCause},
};

pub const PLIC_ADDR: u64 = 0x0c000000;
pub const PLIC_PENDING: u64 = PLIC_ADDR + 0x1000;
pub const PLIC_ENABLE: u64 = PLIC_ADDR + 0x2080;
pub const PLIC_THRESHOLD: u64 = PLIC_ADDR + 0x201000;
pub const PLIC_CLAIM: u64 = PLIC_ADDR + 0x201004;

pub fn plic_check_pending(cpu: &mut Cpu) {
    let plic = &mut cpu.peripherals.as_mut().unwrap().plic;
    let pending = plic.read_mem_u32(PLIC_PENDING).unwrap();
    let enable = plic.read_mem_u32(PLIC_ENABLE).unwrap();
    if 0 != (pending & enable) {
        let sip = cpu.csr_table.read64(CSRAddress::Sip.as_u12())
            | (1 << TrapInterruptCause::SupervisorExternalInterrupt as u64);
        cpu.csr_table.write64(CSRAddress::Sip.as_u12(), sip);
    }
}

pub fn plic_handle_claim_read(cpu: &mut Cpu) -> u32 {
    let plic = &mut cpu.peripherals.as_mut().unwrap().plic;

    let enable = plic.read_mem_u32(PLIC_ENABLE).unwrap();
    let pending = plic.read_mem_u32(PLIC_PENDING).unwrap();
    for i in 0u32..32 {
        let mask = 1u32 << i;
        if (enable & mask) != 0 && (pending & mask) != 0 {
            plic.write_mem_u32(PLIC_CLAIM, i).unwrap();
            plic.write_mem_u32(PLIC_PENDING, pending & !mask).unwrap();
            return i;
        }
    }
    return 0;
}

pub fn plic_handle_claim_write(_cpu: &mut Cpu, _value: u32) {}

pub fn plic_handle_pending_write(_cpu: &mut Cpu, _value: u32) {}

pub fn plic_trigger_irq(cpu: &mut Cpu, irq: u32) {
    let plic = &mut cpu.peripherals.as_mut().unwrap().plic;
    let new_pending = plic.read_mem_u32(PLIC_PENDING).unwrap() | (1 << irq);
    plic.write_mem_u32(PLIC_PENDING, new_pending).unwrap();
}
