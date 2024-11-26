use crate::cpu::cpu_core::{Cpu, PrivilegeMode};

use super::csr::csr_types::{CSRAddress, MstatusCSR};

pub enum TrapCause {
    InstructionAddressMisaligned = 0,
    InstructionAccessFault = 1,
    IllegalInstruction = 2,
    Breakpoint = 3,
    LoadAddressMisaligned = 4,
    LoadAccessFault = 5,
    StoreAddressMisaligned = 6,
    StoreAccessFault = 7,
    EnvironmentCallFromUMode = 8,
    EnvironmentCallFromSMode = 9,
    EnvironmentCallFromMMode = 11,
    InstructionPageFault = 12,
    LoadPageFault = 13,
    StorePageFault = 15,
    SoftwareCheck = 18,
    HadrwareError = 19,
    Unknown = 64,
}

pub fn execute_trap(cpu: &mut Cpu, cause: u64, interrupt: bool) {
    match cpu.arch_mode {
        crate::cpu::cpu_core::CpuMode::RV32 => todo!(),
        crate::cpu::cpu_core::CpuMode::RV64 => {
            let mut mcause: u64 = 0;
            mcause |= cause;
            mcause |= (interrupt as u64) << 63;
            cpu.csr_table.write64(CSRAddress::Mcause.as_u12(), mcause);
        }
    }

    let mut mstatus = MstatusCSR(
        cpu.csr_table
            .read_xlen(CSRAddress::Mstatus.as_u12(), cpu.arch_mode),
    );

    mstatus.set_mpp(cpu.privilege_mode as u64);
    mstatus.set_mpie(mstatus.mie());

    let deleg = match interrupt {
        false => cpu.csr_table.read64(CSRAddress::Medeleg.as_u12()),
        true => cpu.csr_table.read64(CSRAddress::Mideleg.as_u12()),
    };

    let should_delegate = (deleg & (1 << cause)) != 0;

    if should_delegate {
        cpu.privilege_mode = PrivilegeMode::Supervisor;
    } else {
        cpu.privilege_mode = PrivilegeMode::Machine;
    }

    let current_pc = cpu.read_current_instruction_addr_u64();
    cpu.csr_table
        .write_xlen_epc(current_pc, cpu.arch_mode, cpu.privilege_mode);
    let tvec = cpu
        .csr_table
        .read_xlen_tvec(cpu.arch_mode, cpu.privilege_mode);

    let tvec_mode = tvec & 0b11;
    let tvec_base = tvec & !0b11;

    if tvec_mode == 0 {
        cpu.write_pc_u64(tvec_base);
    } else {
        cpu.write_pc_u64(tvec_base + (cause << 2));
    }
}
