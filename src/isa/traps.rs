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
    SupervisorExternalGuestInterrupt = 12
}

pub fn check_pending_interrupts(cpu: &mut Cpu, privilege_mode: PrivilegeMode) {
    let ip_csr = match privilege_mode {
        PrivilegeMode::Machine => CSRAddress::Mip,
        PrivilegeMode::Supervisor => CSRAddress::Sip,
        PrivilegeMode::User => {
            return;
        }
    };
    let ie_csr = match privilege_mode {
        PrivilegeMode::Machine => CSRAddress::Mie,
        PrivilegeMode::Supervisor => CSRAddress::Sie,
        PrivilegeMode::User => {
            return;
        }
    };
    let status_csr = match privilege_mode {
        PrivilegeMode::Machine => CSRAddress::Mstatus,
        PrivilegeMode::Supervisor => CSRAddress::Sstatus,
        PrivilegeMode::User => {
            return;
        }
    };
    let ie = cpu.csr_table.read64(ie_csr.as_u12());
    let ip = cpu.csr_table.read64(ip_csr.as_u12());
    let mstatus = MstatusCSR(
        cpu.csr_table
            .read_xlen(status_csr.as_u12(), cpu.arch_mode),
    );

    let interrupts = ie & ip;
    if interrupts != 0 && mstatus.sie() {
        for i in 0..13 {
            if (interrupts & (1 << i)) != 0 {
                println!("Interrupt: {}", i);
                // clears the interrupt bit in the ip register
                cpu.csr_table.write64(ip_csr.as_u12(), ip & !(1 << i));
                execute_trap(cpu, i, true);
                break;
            }
        }
    }
}

pub fn execute_trap(cpu: &mut Cpu, cause: u64, interrupt: bool) {
    let initial_privilage_mode = cpu.privilege_mode;

    println!(
        "Trap: cause: {}, interrupt: {}",
        cause, interrupt
    );

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

    println!("Privilege mode: {:?} Initial mode: {:?}", cpu.privilege_mode, initial_privilage_mode);

    let status_csr = match cpu.privilege_mode {
        PrivilegeMode::Machine => CSRAddress::Mstatus,
        PrivilegeMode::Supervisor => CSRAddress::Sstatus,
        PrivilegeMode::User => panic!()
    };

    let mut mstatus = MstatusCSR(
        cpu.csr_table
            .read_xlen(status_csr.as_u12(), cpu.arch_mode),
    );

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

    cpu.csr_table.write_xlen(status_csr.as_u12(), mstatus.0, cpu.arch_mode);

    match cpu.arch_mode {
        crate::cpu::cpu_core::CpuMode::RV32 => todo!(),
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

    println!("tvec: 0x{:x}, tvec_mode: {}, tvec_base: 0x{:x}", tvec, tvec_mode, tvec_base);

    if tvec_mode == 0 {
        cpu.write_pc_u64(tvec_base);
    } else {
        cpu.write_pc_u64(tvec_base + (cause << 2));
    }
}
