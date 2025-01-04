use crate::cpu::cpu_core::{Cpu, CpuMode};
use anyhow::{bail, Result};
use bitfield::bitfield;

pub const MMU_PAGE_SIZE: usize = 4096;

pub enum MemoryMode {
    Bare,
    Sv32,
    Sv39,
    Sv48,
    Sv57,
}

bitfield! {
    #[allow(non_camel_case_types)]
    pub struct Sv39_PTE(u64);
    impl Debug;
    pub v, set_v: 0;            // Valid
    pub r, set_r: 1;            // Readable
    pub w, set_w: 2;            // Writable
    pub x, set_x: 3;            // Executable
    pub u, set_u: 4;            // User accessible
    pub g, set_g: 5;            // Global
    pub a, set_a: 6;            // Accessed
    pub d, set_d: 7;            // Dirty
    pub rsw, set_rsw: 9, 8;     // Reserved for Software
    pub ppn, set_ppn: 53, 10;   // PPN bits
    pub ppn0, set_ppn0: 18, 10;   // PPN[0] bits
    pub ppn1, set_ppn1: 27, 19;   // PPN[1] bits
    pub ppn2, set_ppn2: 53, 28;   // PPN[2] bits

}

bitfield! {
    #[allow(non_camel_case_types)]
    pub struct Sv39_VirtualAddress(u64);
    impl Debug;
    pub offset, set_offset: 11, 0;   // VPN[0] bits
    pub vpn0, set_vpn0: 20, 12;  // VPN[1] bits
    pub vpn1, set_vpn1: 29, 21;  // VPN[2] bits
    pub vpn2, set_vpn2: 38, 30;  // VPN[3] bits
}

bitfield! {
    #[allow(non_camel_case_types)]
    pub struct Sv39_PhysicalAddress(u64);
    impl Debug;
    pub offset, set_offset: 11, 0;  // Offset bits
    pub ppn, set_ppn: 55, 12;     // PPN bits
    pub ppn0, set_ppn0: 20, 12;   // PPN[0] bits
    pub ppn1, set_ppn1: 29, 21;   // PPN[1] bits
    pub ppn2, set_ppn2: 55, 30;   // PPN[2] bits
}

pub fn read_memory_mode(csr: u64, cpu_mode: CpuMode) -> MemoryMode {
    match cpu_mode {
        CpuMode::RV32 => match csr {
            0 => MemoryMode::Bare,
            1 => MemoryMode::Sv32,
            _ => panic!("Invalid memory mode"),
        },
        CpuMode::RV64 => match csr {
            0 => MemoryMode::Bare,
            8 => MemoryMode::Sv39,
            9 => MemoryMode::Sv48,
            10 => MemoryMode::Sv57,
            _ => panic!("Invalid memory mode"),
        },
    }
}

pub fn read_root_page_table_pointer(satp: u64, cpu_mode: CpuMode) -> u64 {
    match cpu_mode {
        CpuMode::RV32 => {
            todo!()
        }
        CpuMode::RV64 => (satp & ((1 << 44) - 1)) << 12,
    }
}

pub fn walk_page_table_sv39(va: u64, satp: u64, cpu: &mut Cpu) -> Result<u64> {
    let virtual_address = Sv39_VirtualAddress(va);
    let vpn0 = virtual_address.vpn0();
    let vpn1 = virtual_address.vpn1();
    let vpn2 = virtual_address.vpn2();
    let offset = virtual_address.offset();

    let l2_page_table_addr = read_root_page_table_pointer(satp, CpuMode::RV64);
    let l2_pte = Sv39_PTE(cpu.memory.read_mem_u64(l2_page_table_addr + vpn2 * 8)?);
    // Check V bit
    if !l2_pte.v() {
        bail!(
            "Invalid L2 PTE, virtual address: {:#x}, addr: {:#x}\n{:?}",
            va,
            l2_page_table_addr + vpn2 * 8,
            l2_pte
        );
    }
    if l2_pte.x() || l2_pte.r() {
        panic!();
        // leaf 1G
        let mut physical_address = Sv39_PhysicalAddress(0);
        physical_address.set_offset(offset);
        physical_address.set_ppn0(vpn0);
        physical_address.set_ppn1(vpn1);
        physical_address.set_ppn2(l2_pte.ppn2());
        return Ok(physical_address.0);
    }

    let l1_page_table_addr = l2_pte.ppn() << 12;
    let l1_pte = Sv39_PTE(cpu.memory.read_mem_u64(l1_page_table_addr + vpn1 * 8)?);
    if !l1_pte.v() {
        bail!(
            "Invalid L1 PTE, virtual address: {:#x}, addr: {:#x}\n{:?}",
            va,
            l1_page_table_addr + vpn1 * 8,
            l1_pte
        );
    }
    if l1_pte.x() || l1_pte.r() {
        panic!();
        // leaf 2MB
        let mut physical_address = Sv39_PhysicalAddress(0);
        physical_address.set_offset(offset);
        physical_address.set_ppn0(vpn0);
        physical_address.set_ppn1(l1_pte.ppn1());
        physical_address.set_ppn2(l1_pte.ppn2());
        return Ok(physical_address.0);
    }

    let l0_page_table_addr = l1_pte.ppn() << 12;
    let l0_pte = Sv39_PTE(cpu.memory.read_mem_u64(l0_page_table_addr + vpn0 * 8)?);
    if !l0_pte.v() {
        println!("Root page addr: {:#x}", l2_page_table_addr);
        println!(
            "L2 PTE: {:#x} {:#x}",
            l2_pte.0,
            l2_page_table_addr + vpn2 * 8
        );
        println!(
            "L1 PTE: {:#x} {:#x}",
            l1_pte.0,
            l1_page_table_addr + vpn1 * 8
        );
        println!(
            "L0 PTE: {:#x} {:#x}",
            l0_pte.0,
            l0_page_table_addr + vpn0 * 8
        );
        println!("L2 PTE: {:?}", l2_pte);
        println!("L1 PTE: {:?}", l1_pte);
        println!("L0 PTE: {:?}", l0_pte);
        println!("VA: {:x}", virtual_address.0);
        println!("PC: {:x}", cpu.read_pc());
        println!("Offset: {:x}", offset);
        println!("l2_page_addr: {:x}", l2_page_table_addr);
        println!("l1_page_addr: {:x}", l1_page_table_addr);
        println!("l0_page_addr: {:x}", l0_page_table_addr);
        let bt = std::backtrace::Backtrace::capture();
        println!("{}", bt);
        bail!(
            "Invalid L0 PTE, virtual address: {:#x}, addr: {:#x}, pte: {:#x}",
            va,
            l0_page_table_addr + vpn0 * 8,
            l0_pte.0
        );
    }

    let physical_address = Sv39_PhysicalAddress(l0_pte.ppn() << 12 | offset); // 4KB

    Ok(physical_address.0)
}
