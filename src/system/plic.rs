use crate::{
    cpu::cpu_core::Cpu,
    isa::{
        csr::csr_types::{CSRAddress, MstatusCSR},
        traps::TrapInterruptCause,
    },
};

pub const PLIC_ADDR: u64 = 0x0c000000;
pub const PLIC_PENDING: u64 = PLIC_ADDR + 0x1000;
pub const PLIC_ENABLE: u64 = PLIC_ADDR + 0x2080;
pub const PLIC_THRESHOLD: u64 = PLIC_ADDR + 0x201000;
pub const PLIC_CLAIM: u64 = PLIC_ADDR + 0x201004;

pub fn plic_check_pending(cpu: &mut Cpu) {
    let pending = cpu.read_mem_u32(PLIC_PENDING).unwrap();
    let enable = cpu.read_mem_u32(PLIC_ENABLE).unwrap();
    if 0 != (pending & enable) {
        //cpu.write_mem_u32(PLIC_PENDING, 0).unwrap(); // TODO: verify behavior

        let sip = cpu.csr_table.read64(CSRAddress::Sip.as_u12())
            | (1 << TrapInterruptCause::SupervisorExternalInterrupt as u64);
        cpu.csr_table.write64(CSRAddress::Sip.as_u12(), sip);
        //println!(
        //    "plic_check_pending: pending={:#x}, enable={:#x}",
        //    pending, enable
        //);

        let sie = cpu.csr_table.read64(CSRAddress::Sie.as_u12());
        let sip = cpu.csr_table.read64(CSRAddress::Sip.as_u12());
        //println!(
        //    "plic_check_pending: sie={:#x}, sip={:#x}, and:{:#x}",
        //    sie,
        //    sip,
        //    sip & sie
        //);

        let sstatus = MstatusCSR(cpu.csr_table.read64(CSRAddress::Sstatus.as_u12()));
        //println!("plic_check_pending: sstatus={:?}", sstatus);
        //
        //println!("current mode={:?}", cpu.privilege_mode);
    }
}

pub fn plic_handle_claim_read(cpu: &mut Cpu) -> u32 {
    println!("plic_handle_claim_read");
    let enable = cpu.read_mem_u32(PLIC_ENABLE).unwrap();
    let pending = cpu.read_mem_u32(PLIC_PENDING).unwrap();
    println!(
        "plic_handle_claim_read: enable={:#x}, pending={:#x}",
        enable, pending
    );
    for i in 0u32..32 {
        let mask = 1u32 << i;
        if (enable & mask) != 0 && (pending & mask) != 0 {
            println!("plic_handle_claim_read: claim={}", i);
            cpu.write_mem_u32(PLIC_CLAIM, i).unwrap();
            cpu.write_mem_u32(PLIC_PENDING, pending & !mask).unwrap();
            return i;
        }
    }
    return 0;
}

pub fn plic_handle_claim_write(_cpu: &mut Cpu, value: u32) {
    println!(
        "plic_handle_claim_write 0x{:x} from {:x}",
        value,
        _cpu.read_current_instruction_addr_u64()
    );
    // TODO: check if value is valid
}

pub fn plic_handle_pending_write(_cpu: &mut Cpu, value: u32) {
    println!(
        "plic_handle_pending_write 0x{:x} from {:x}",
        value,
        _cpu.read_current_instruction_addr_u64()
    );
    let bt = std::backtrace::Backtrace::capture();
    println!("{}", bt);
    // TODO: check if value is valid
}

pub fn plic_trigger_irq(cpu: &mut Cpu, irq: u32) {
    let new_pending = cpu.read_mem_u32(PLIC_PENDING).unwrap() | (1 << irq);
    cpu.write_mem_u32(PLIC_PENDING, new_pending).unwrap();
    println!("plic_trigger_irq: irq={}, pending={:x}", irq, new_pending);
}
