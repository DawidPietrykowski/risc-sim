use crate::types::{BitValue, U12};
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
    // ... TODO

    // Machine Counter/Timers
    Mcycle = 0xB00,
    Minstret = 0xB02,
    Mhpmcounter3 = 0xB03,
    Mhpmcounter4 = 0xB04,
    // ... TODO

    // Machine Counter Setup
    Mcountinhibit = 0x320,
    Mhpmevent3 = 0x323,
    Mhpmevent4 = 0x324,
}

pub struct CSRTable {
    pub csrs: [u32; 4096],
}

impl CSRTable {
    pub fn new() -> Self {
        CSRTable { csrs: [0; 4096] }
    }

    pub fn read(&self, addr: U12) -> u32 {
        self.csrs[addr.value() as usize]
    }

    pub fn write(&mut self, addr: U12, value: u32) {
        self.csrs[addr.value() as usize] = value;
    }
}
