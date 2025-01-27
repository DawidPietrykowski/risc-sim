use crate::system::{
    plic::{
        plic_handle_claim_read, plic_handle_claim_write, plic_handle_pending_write, PLIC_CLAIM,
        PLIC_PENDING,
    },
    uart::{uart_handle_read, uart_handle_write, UART_ADDR},
    virtio::{process_queue, VIRTIO_0_ADDR, VIRTIO_MMIO_QUEUE_NOTIFY},
};
use anyhow::Result;

use super::{
    cpu_core::{Cpu, KERNEL_ADDR},
    memory::memory_core::Memory,
};

pub(crate) fn bare_read_mem_u64(cpu: &mut Cpu, addr: u64) -> Result<u64> {
    let addr = cpu.translate_address_if_needed(addr)?;
    cpu.memory.read_mem_u64(addr)
}

pub(crate) fn bare_read_mem_u32(cpu: &mut Cpu, addr: u64) -> Result<u32> {
    let addr = cpu.translate_address_if_needed(addr)?;
    if addr <= KERNEL_ADDR {
        if addr < UART_ADDR {
            // PLIC
            if addr == PLIC_CLAIM {
                return Ok(plic_handle_claim_read(cpu));
            }
            return cpu.peripherals.as_mut().unwrap().plic.read_mem_u32(addr);
        } else if addr < VIRTIO_0_ADDR {
            // UART
            return cpu.peripherals.as_mut().unwrap().uart.read_mem_u32(addr);
        } else {
            // VIRTIO
            return cpu.peripherals.as_mut().unwrap().virtio.read_mem_u32(addr);
        }
    }
    cpu.memory.read_mem_u32(addr)
}

pub(crate) fn bare_read_mem_u16(cpu: &mut Cpu, addr: u64) -> Result<u16> {
    let addr = cpu.translate_address_if_needed(addr)?;
    cpu.memory.read_mem_u16(addr)
}

pub(crate) fn bare_read_mem_u8(cpu: &mut Cpu, addr: u64) -> Result<u8> {
    let addr = cpu.translate_address_if_needed(addr)?;
    if addr <= KERNEL_ADDR {
        if addr < UART_ADDR {
            // PLIC
            return cpu.peripherals.as_mut().unwrap().plic.read_mem_u8(addr);
        } else if addr < VIRTIO_0_ADDR {
            // UART
            if addr == UART_ADDR {
                return Ok(uart_handle_read(cpu) as u8);
            }
            return cpu.peripherals.as_mut().unwrap().uart.read_mem_u8(addr);
        } else {
            // VIRTIO
            return cpu.peripherals.as_mut().unwrap().virtio.read_mem_u8(addr);
        }
    }
    cpu.memory.read_mem_u8(addr)
}

pub(crate) fn bare_write_mem_u8(cpu: &mut Cpu, addr: u64, value: u8) -> Result<()> {
    let addr = cpu.translate_address_if_needed(addr)?;
    if addr <= KERNEL_ADDR {
        if addr < UART_ADDR {
            // PLIC
            return cpu
                .peripherals
                .as_mut()
                .unwrap()
                .plic
                .write_mem_u8(addr, value);
        } else if addr < VIRTIO_0_ADDR {
            // UART
            if addr == UART_ADDR {
                uart_handle_write(cpu, value);
                return Ok(());
            }
            return cpu
                .peripherals
                .as_mut()
                .unwrap()
                .uart
                .write_mem_u8(addr, value);
        } else {
            // VIRTIO
            return cpu
                .peripherals
                .as_mut()
                .unwrap()
                .virtio
                .write_mem_u8(addr, value);
        }
    }
    cpu.memory.write_mem_u8(addr, value)
}

pub(crate) fn bare_write_mem_u16(cpu: &mut Cpu, addr: u64, value: u16) -> Result<()> {
    let addr = cpu.translate_address_if_needed(addr)?;
    cpu.memory.write_mem_u16(addr, value)
}

pub(crate) fn bare_write_mem_u32(cpu: &mut Cpu, addr: u64, value: u32) -> Result<()> {
    let addr = cpu.translate_address_if_needed(addr)?;
    if addr <= KERNEL_ADDR {
        if addr < UART_ADDR {
            // PLIC
            if addr == PLIC_PENDING {
                plic_handle_pending_write(cpu, value);
                return Ok(());
            }
            if addr == PLIC_CLAIM {
                plic_handle_claim_write(cpu, value);
                return Ok(());
            }
            return cpu
                .peripherals
                .as_mut()
                .unwrap()
                .plic
                .write_mem_u32(addr, value);
        } else if addr < VIRTIO_0_ADDR {
            // UART
            return cpu
                .peripherals
                .as_mut()
                .unwrap()
                .uart
                .write_mem_u32(addr, value);
        } else {
            // VIRTIO
            if addr == VIRTIO_0_ADDR + VIRTIO_MMIO_QUEUE_NOTIFY as u64 {
                process_queue(cpu);
            }
            return cpu
                .peripherals
                .as_mut()
                .unwrap()
                .virtio
                .write_mem_u32(addr, value);
        }
    }
    cpu.memory.write_mem_u32(addr, value)
}

pub(crate) fn bare_write_mem_u64(cpu: &mut Cpu, addr: u64, value: u64) -> Result<()> {
    let addr = cpu.translate_address_if_needed(addr)?;
    cpu.memory.write_mem_u64(addr, value)
}

pub(crate) fn user_space_read_mem_u64(cpu: &mut Cpu, addr: u64) -> Result<u64> {
    cpu.memory.read_mem_u64(addr)
}

pub(crate) fn user_space_read_mem_u32(cpu: &mut Cpu, addr: u64) -> Result<u32> {
    cpu.memory.read_mem_u32(addr)
}

pub(crate) fn user_space_read_mem_u16(cpu: &mut Cpu, addr: u64) -> Result<u16> {
    cpu.memory.read_mem_u16(addr)
}

pub(crate) fn user_space_read_mem_u8(cpu: &mut Cpu, addr: u64) -> Result<u8> {
    cpu.memory.read_mem_u8(addr)
}

pub(crate) fn user_space_write_mem_u8(cpu: &mut Cpu, addr: u64, value: u8) -> Result<()> {
    cpu.memory.write_mem_u8(addr, value)
}

pub(crate) fn user_space_write_mem_u16(cpu: &mut Cpu, addr: u64, value: u16) -> Result<()> {
    cpu.memory.write_mem_u16(addr, value)
}

pub(crate) fn user_space_write_mem_u32(cpu: &mut Cpu, addr: u64, value: u32) -> Result<()> {
    cpu.memory.write_mem_u32(addr, value)
}

pub(crate) fn user_space_write_mem_u64(cpu: &mut Cpu, addr: u64, value: u64) -> Result<()> {
    cpu.memory.write_mem_u64(addr, value)
}
