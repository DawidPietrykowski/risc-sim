use crate::{cpu::cpu_core::Cpu, isa::{csr::csr_types::CSRAddress, traps::TrapInterruptCause}};

pub const PLIC_ADDR: u64 = 0x0c000000;
pub const PLIC_PENDING: u64 = PLIC_ADDR + 0x1000;
pub const PLIC_ENABLE: u64 = PLIC_ADDR + 0x2080;
pub const PLIC_THRESHOLD: u64 = PLIC_ADDR + 0x201000;
pub const PLIC_CLAIM: u64 = PLIC_ADDR + 0x201004;

pub fn plic_check_pending(cpu: &mut Cpu) {
    let pending = cpu.read_mem_u32(PLIC_PENDING).unwrap();
    if 0 != pending {
        cpu.write_mem_u32(PLIC_PENDING, 0).unwrap(); // TODO: verify behavior

        let sip = cpu.csr_table.read64(CSRAddress::Sip.as_u12()) | (1 << TrapInterruptCause::SupervisorExternalInterrupt as u64);
        cpu.csr_table.write64(CSRAddress::Sip.as_u12(), sip);
        println!("plic_check_pending: pending={:#x}", pending);

        let sie = cpu.csr_table.read64(CSRAddress::Sie.as_u12());
        let sip = cpu.csr_table.read64(CSRAddress::Sip.as_u12());
        println!("plic_check_pending: sie={:#x}, sip={:#x}", sie, sip);

        let sstatus = cpu.csr_table.read64(CSRAddress::Sstatus.as_u12());
        println!("plic_check_pending: sstatus={:#x}", sstatus);

        println!("current mode={:?}", cpu.privilege_mode);
    }
}

pub fn plic_handle_claim_read(cpu: &mut Cpu) {
    println!("plic_handle_claim_read");
    let enable = cpu.read_mem_u32(PLIC_ENABLE).unwrap();
    let pending = cpu.read_mem_u32(PLIC_PENDING).unwrap();
    for i in 0u32..32 {
        let mask = 1u32 << i;
        if (enable & mask) != 0 && (pending & mask) != 0 {
            cpu.write_mem_u32(PLIC_CLAIM, i).unwrap();
            cpu.write_mem_u32(PLIC_PENDING, pending & !mask).unwrap();
            return;
        }
    }
}

pub fn plic_handle_claim_write(_cpu: &mut Cpu, _value: u32) {
    println!("plic_handle_claim_write");
    // TODO: check if value is valid
}

pub fn plic_trigger_irq(cpu: &mut Cpu, irq: u32) {
    let new_pending = cpu.read_mem_u32(PLIC_PENDING).unwrap() | (1 << irq);
    cpu.write_mem_u32(PLIC_PENDING, new_pending).unwrap();
    println!("plic_trigger_irq: irq={}, pending={:x}", irq, new_pending);
}