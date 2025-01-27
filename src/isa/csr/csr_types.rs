use crate::{
    cpu::cpu_core::{CpuMode, PrivilegeMode},
    types::{BitValue, U12},
};
use bitfield::bitfield;

// Supervisor status register (sstatus) bit positions
pub const SSTATUS_SIE: u64 = 1 << 1; // Supervisor Interrupt Enable
pub const SSTATUS_SPIE: u64 = 1 << 5; // Previous SIE
pub const SSTATUS_SPP: u64 = 1 << 8; // Previous privilege mode
pub const SSTATUS_FS: u64 = 3 << 13; // FPU status
pub const SSTATUS_XS: u64 = 3 << 15; // Extension status
pub const SSTATUS_SUM: u64 = 1 << 18; // Supervisor User Memory access
pub const SSTATUS_MXR: u64 = 1 << 19; // Make eXecutable Readable
pub const SSTATUS_SD: u64 = 1 << 63; // State Dirty (read-only)

pub const SSTATUS_MASK: u64 = SSTATUS_SIE
    | SSTATUS_SPIE
    | SSTATUS_SPP
    | SSTATUS_FS
    | SSTATUS_XS
    | SSTATUS_SUM
    | SSTATUS_MXR
    | SSTATUS_SD;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CSRAddress {
    // User Trap Setup
    Ustatus = 0x000,
    Uie = 0x004,
    Utvec = 0x005,

    // User Trap Handling
    Uscratch = 0x040,
    Uepc = 0x041,
    Ucause = 0x042,
    Utval = 0x043,
    Uip = 0x044,

    // User Floating-Point CSRs
    Fflags = 0x001,
    Frm = 0x002,
    Fcsr = 0x003,

    // User Counter/Timers
    Cycle = 0xC00,
    CycleH = 0xC80,
    Time = 0xC01,
    TimeH = 0xC81,
    Instret = 0xC02,
    InstretH = 0xC82,

    // Supervisor Trap Setup
    Sstatus = 0x100,
    Sedeleg = 0x102,
    Sideleg = 0x103,
    Sie = 0x104,
    Stvec = 0x105,
    Scounteren = 0x106,

    // Supervisor Trap Handling
    Sscratch = 0x140,
    Sepc = 0x141,
    Scause = 0x142,
    Stval = 0x143,
    Sip = 0x144,

    // Supervisor Timers
    Stimecmp = 0x14D,

    // Supervisor Protection and Translation
    Satp = 0x180,

    // Machine Information Registers
    Mvendorid = 0xF11,
    Marchid = 0xF12,
    Mimpid = 0xF13,
    Mhartid = 0xF14,

    // Machine Trap Setup
    Mstatus = 0x300,
    Misa = 0x301,
    Medeleg = 0x302,
    Mideleg = 0x303,
    Mie = 0x304,
    Mtvec = 0x305,
    Mcounteren = 0x306,

    // Machine Trap Handling
    Mscratch = 0x340,
    Mepc = 0x341,
    Mcause = 0x342,
    Mtval = 0x343,
    Mip = 0x344,

    // Machine Protection and Translation
    Pmpcfg0 = 0x3A0,
    Pmpcfg1 = 0x3A1,
    Pmpcfg2 = 0x3A2,
    Pmpcfg3 = 0x3A3,
    Pmpaddr0 = 0x3B0,
    Pmpaddr1 = 0x3B1,

    // Machine Counter/Timers
    Mcycle = 0xB00,
    Minstret = 0xB02,
    Mhpmcounter3 = 0xB03,
    Mhpmcounter4 = 0xB04,

    // Machine Counter Setup
    Mcountinhibit = 0x320,
    Mhpmevent3 = 0x323,
    Mhpmevent4 = 0x324,
}

bitfield! {
    pub struct MisaCSR(u64);
    impl Debug;
    pub mxl_32, set_mxl_32: 31, 30;
    pub mxl_64, set_mxl_64: 63, 62;
    pub extension_a, set_extension_a: 0;
    pub extension_c, set_extension_c: 2;
    pub extension_d, set_extension_d: 3;
    pub extension_f, set_extension_f: 5;
    pub extension_i, set_extension_i: 8;
    pub extension_m, set_extension_m: 12;
    pub extension_s, set_extension_s: 18;
    pub extension_u, set_extension_u: 20;
}

bitfield! {
    pub struct MstatusCSR(u64);
    impl Debug;
    pub sie, set_sie: 1;
    pub mie, set_mie: 3;
    pub spie, set_spie: 5;
    pub ube, set_ube: 6;
    pub mpie, set_mpie: 7;
    pub spp, set_spp: 8;
    pub vs, set_vs: 10, 9;
    pub mpp, set_mpp: 12, 11;
    pub fs, set_fs: 14, 13;
    pub xs, set_xs: 16, 15;
    pub mprv, set_mprv: 17;
    pub sum, set_sum: 18;
    pub mxr, set_mxr: 19;
    pub tvm, set_tvm: 20;
    pub tw, set_tw: 21;
    pub tsr, set_tsr: 22;
    pub sd_32, set_sd_32: 31;
    pub sd_64, set_sd_64: 63;
}

impl CSRAddress {
    pub fn as_u12(self) -> U12 {
        U12::new(self as u16)
    }
}

pub struct CSRTable {
    pub csrs32: [u32; 4096],
    pub csrs64: [u64; 4096],
}

impl CSRTable {
    pub fn new(cpu_mode: CpuMode) -> Self {
        let mut csr_table = CSRTable {
            csrs32: [0; 4096],
            csrs64: [0; 4096],
        };

        let mut misa = MisaCSR(0);
        misa.set_extension_i(true);
        misa.set_extension_m(true);
        match cpu_mode {
            CpuMode::RV32 => {
                misa.set_mxl_32(1);
                csr_table.write32(CSRAddress::Misa.as_u12(), misa.0 as u32);
            }
            CpuMode::RV64 => {
                misa.set_mxl_64(2);
                csr_table.write64(CSRAddress::Misa.as_u12(), misa.0);
            }
        }
        csr_table.write32(CSRAddress::Mvendorid.as_u12(), 0);
        csr_table.write_xlen(CSRAddress::Mhartid.as_u12(), 0, cpu_mode);

        csr_table
    }

    pub fn read32(&self, addr: U12) -> u32 {
        self.csrs32[addr.value() as usize]
    }

    pub fn write32(&mut self, addr: U12, value: u32) {
        self.csrs32[addr.value() as usize] = value;
    }

    pub fn read64(&self, addr: U12) -> u64 {
        if addr == CSRAddress::Sie.as_u12() {
            let mideleg = self.read64(CSRAddress::Mideleg.as_u12());
            let mie = self.read64(CSRAddress::Mie.as_u12());
            return mie & mideleg;
        }
        if addr == CSRAddress::Sip.as_u12() {
            let mideleg = self.read64(CSRAddress::Mideleg.as_u12());
            let mip = self.read64(CSRAddress::Mip.as_u12());
            return mip & mideleg;
        }
        if addr == CSRAddress::Sstatus.as_u12() {
            let mstatus = self.read64(CSRAddress::Mstatus.as_u12());
            return mstatus & SSTATUS_MASK;
        }
        self.csrs64[addr.value() as usize]
    }

    pub fn write64(&mut self, addr: U12, value: u64) {
        if addr == CSRAddress::Sie.as_u12() {
            let mideleg = self.read64(CSRAddress::Mideleg.as_u12());
            let mie = self.read64(CSRAddress::Mie.as_u12());
            self.write64(
                CSRAddress::Mie.as_u12(),
                (mie & !mideleg) | (value & mideleg),
            );
            return;
        }
        if addr == CSRAddress::Sip.as_u12() {
            let mideleg = self.read64(CSRAddress::Mideleg.as_u12());
            let mip = self.read64(CSRAddress::Mip.as_u12());
            self.write64(
                CSRAddress::Mip.as_u12(),
                (mip & !mideleg) | (value & mideleg),
            );
            return;
        }
        if addr == CSRAddress::Sstatus.as_u12() {
            let mstatus = self.read64(CSRAddress::Mstatus.as_u12());
            self.write64(
                CSRAddress::Mstatus.as_u12(),
                (mstatus & !SSTATUS_MASK) | (value & SSTATUS_MASK),
            );
            return;
        }
        self.csrs64[addr.value() as usize] = value;
    }

    pub fn write_xlen(&mut self, addr: U12, value: u64, mode: CpuMode) {
        match mode {
            CpuMode::RV32 => self.write32(addr, value as u32),
            CpuMode::RV64 => self.write64(addr, value),
        }
    }

    pub fn read_xlen(&self, addr: U12, mode: CpuMode) -> u64 {
        match mode {
            CpuMode::RV32 => self.read32(addr) as u64,
            CpuMode::RV64 => self.read64(addr),
        }
    }

    pub fn read_xlen_tvec(&self, mode: CpuMode, privilege_mode: PrivilegeMode) -> u64 {
        match privilege_mode {
            PrivilegeMode::Machine => self.read_xlen(CSRAddress::Mtvec.as_u12(), mode),
            PrivilegeMode::Supervisor => self.read_xlen(CSRAddress::Stvec.as_u12(), mode),
            PrivilegeMode::User => panic!(),
        }
    }

    pub fn write_xlen_tvec(&mut self, value: u64, mode: CpuMode, privilege_mode: PrivilegeMode) {
        match privilege_mode {
            PrivilegeMode::Machine => self.write_xlen(CSRAddress::Mtvec.as_u12(), value, mode),
            PrivilegeMode::Supervisor => self.write_xlen(CSRAddress::Stvec.as_u12(), value, mode),
            PrivilegeMode::User => panic!(),
        }
    }

    pub fn read_xlen_tval(&self, mode: CpuMode, privilege_mode: PrivilegeMode) -> u64 {
        match privilege_mode {
            PrivilegeMode::Machine => self.read_xlen(CSRAddress::Mtval.as_u12(), mode),
            PrivilegeMode::Supervisor => self.read_xlen(CSRAddress::Stval.as_u12(), mode),
            PrivilegeMode::User => panic!(),
        }
    }

    pub fn write_xlen_tval(&mut self, value: u64, mode: CpuMode, privilege_mode: PrivilegeMode) {
        match privilege_mode {
            PrivilegeMode::Machine => self.write_xlen(CSRAddress::Mtval.as_u12(), value, mode),
            PrivilegeMode::Supervisor => self.write_xlen(CSRAddress::Stval.as_u12(), value, mode),
            PrivilegeMode::User => panic!(),
        }
    }

    pub fn read_xlen_status(&self, mode: CpuMode, privilege_mode: PrivilegeMode) -> u64 {
        match privilege_mode {
            PrivilegeMode::Machine => self.read_xlen(CSRAddress::Mstatus.as_u12(), mode),
            PrivilegeMode::Supervisor => self.read_xlen(CSRAddress::Sstatus.as_u12(), mode),
            PrivilegeMode::User => self.read_xlen(CSRAddress::Ustatus.as_u12(), mode),
        }
    }

    pub fn write_xlen_epc(&mut self, value: u64, mode: CpuMode, privilege_mode: PrivilegeMode) {
        match privilege_mode {
            PrivilegeMode::Machine => self.write_xlen(CSRAddress::Mepc.as_u12(), value, mode),
            PrivilegeMode::Supervisor => self.write_xlen(CSRAddress::Sepc.as_u12(), value, mode),
            PrivilegeMode::User => panic!(),
        }
    }

    pub fn read_xlen_epc(&self, mode: CpuMode, privilege_mode: PrivilegeMode) -> u64 {
        match privilege_mode {
            PrivilegeMode::Machine => self.read_xlen(CSRAddress::Mepc.as_u12(), mode),
            PrivilegeMode::Supervisor => self.read_xlen(CSRAddress::Sepc.as_u12(), mode),
            PrivilegeMode::User => panic!(),
        }
    }
}

impl Default for CSRTable {
    fn default() -> Self {
        Self::new(CpuMode::RV32)
    }
}
