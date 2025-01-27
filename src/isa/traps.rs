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

pub enum TrapInterruptCause {
    SupervisorSoftwareInterrupt = 1,
    MachineSoftwareInterrupt = 3,
    SupervisorTimerInterrupt = 5,
    MachineTimerInterrupt = 7,
    SupervisorExternalInterrupt = 9,
    MachineExternalInterrupt = 11,
    SupervisorExternalGuestInterrupt = 12,
}

pub fn update_timers(cpu: &mut Cpu) {
    cpu.csr_table.write64(
        CSRAddress::Time.as_u12(),
        cpu.csr_table.read64(CSRAddress::Time.as_u12()) + 1,
    );
}

pub fn check_pending_interrupts(cpu: &mut Cpu) {
    if cpu.csr_table.read64(CSRAddress::Time.as_u12())
        > cpu.csr_table.read64(CSRAddress::Stimecmp.as_u12())
    {
        let mip = cpu.csr_table.read64(CSRAddress::Mip.as_u12());

        const STIP_BIT_POS: u64 = 5;

        cpu.csr_table
            .write64(CSRAddress::Mip.as_u12(), mip | (1 << STIP_BIT_POS));
    }
    let mip_addr = CSRAddress::Mip.as_u12();
    let sip_addr = CSRAddress::Sip.as_u12();
    let mie_addr = CSRAddress::Mie.as_u12();
    let sie_addr = CSRAddress::Sie.as_u12();
    let mideleg = cpu.csr_table.read64(CSRAddress::Mideleg.as_u12());

    let mip = cpu.csr_table.read64(mip_addr);
    let sip = cpu.csr_table.read64(sip_addr);
    let mie_csr = cpu.csr_table.read64(mie_addr);
    let sie = cpu.csr_table.read64(sie_addr);

    let pending = mip & mie_csr;
    let spending = pending & sie & mideleg;

    let mstatus = MstatusCSR(cpu.csr_table.read64(CSRAddress::Mstatus.as_u12()));
    let mie = mstatus.mie();
    let sie = mstatus.sie();

    if pending != 0 && mie {
        for i in 0..13 {
            if (pending & (1 << i)) != 0 {
                // clears the interrupt bit in the ip register
                cpu.csr_table.write64(mip_addr, mip & !(1 << i));
                execute_trap(cpu, i, true);
                break;
            }
        }
    } else if spending != 0 && sie {
        for i in 0..13 {
            if (spending & (1 << i)) != 0 {
                // clears the interrupt bit in the ip register
                cpu.csr_table.write64(sip_addr, sip & !(1 << i));
                execute_trap(cpu, i, true);
                break;
            }
        }
    }
}

pub fn execute_trap(cpu: &mut Cpu, cause: u64, interrupt: bool) {
    let initial_privilage_mode = cpu.privilege_mode;

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

    let status_csr = match cpu.privilege_mode {
        PrivilegeMode::Machine => CSRAddress::Mstatus,
        PrivilegeMode::Supervisor => CSRAddress::Sstatus,
        PrivilegeMode::User => panic!(),
    };

    let mut mstatus = MstatusCSR(cpu.csr_table.read_xlen(status_csr.as_u12(), cpu.arch_mode));

    if cpu.privilege_mode == PrivilegeMode::Machine {
        mstatus.set_mpp(initial_privilage_mode as u64);
        mstatus.set_mpie(mstatus.mie());
        mstatus.set_mie(false);
    } else {
        if initial_privilage_mode == PrivilegeMode::User {
            mstatus.set_spp(false);
        } else if initial_privilage_mode == PrivilegeMode::Supervisor {
            mstatus.set_spp(true);
        } else {
            panic!("Invalid privilege mode");
        }
        mstatus.set_spie(mstatus.sie());
        mstatus.set_sie(false);
    }

    cpu.csr_table
        .write_xlen(status_csr.as_u12(), mstatus.0, cpu.arch_mode);

    match cpu.arch_mode {
        crate::cpu::cpu_core::CpuMode::RV32 => unimplemented!(),
        crate::cpu::cpu_core::CpuMode::RV64 => {
            let mut mcause: u64 = 0;
            mcause |= cause;
            mcause |= (interrupt as u64) << 63;
            if cpu.privilege_mode == PrivilegeMode::Supervisor {
                cpu.csr_table.write64(CSRAddress::Scause.as_u12(), mcause);
            } else {
                cpu.csr_table.write64(CSRAddress::Mcause.as_u12(), mcause);
            }
        }
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
