use core::slice::SlicePattern;
use std::{
    fs::File,
    io::{Read, Write},
};

use crate::{
    cpu::{cpu_core::Cpu, memory::memory_core::Memory},
    system::plic::plic_trigger_irq,
};

const VIRTIO_DESC_NUM: usize = 8;

pub const VIRTIO_0_ADDR: u64 = 0x10001000;

pub const VIRTIO_MMIO_MAGIC_VALUE: u32 = 0x000;
pub const VIRTIO_MMIO_VERSION: u32 = 0x004;
pub const VIRTIO_MMIO_DEVICE_ID: u32 = 0x008;
pub const VIRTIO_MMIO_VENDOR_ID: u32 = 0x00c;

const VIRTIO_MMIO_QUEUE_DESC_LOW: u32 = 0x080;
const VIRTIO_MMIO_QUEUE_DESC_HIGH: u32 = 0x084;
const VIRTIO_MMIO_QUEUE_AVAIL_LOW: u32 = 0x090;
const VIRTIO_MMIO_QUEUE_AVAIL_HIGH: u32 = 0x094;
const VIRTIO_MMIO_QUEUE_USED_LOW: u32 = 0x0a0;
const VIRTIO_MMIO_QUEUE_USED_HIGH: u32 = 0x0a4;

const VIRTIO_MMIO_QUEUE_NUM_MAX: u32 = 0x034;

pub const VIRTIO_MMIO_QUEUE_NOTIFY: u32 = 0x050;

const VRING_DESC_F_NEXT: u16 = 1;
const VRING_DESC_F_WRITE: u16 = 2;

const VIRTIO0_IRQ: u32 = 1;

const VIRTIO_BLK_T_IN: u32 = 0;
const VIRTIO_BLK_T_OUT: u32 = 1;
const VIRTIO_BLK_S_OK: u8 = 0;

#[repr(u32)]
#[derive(Clone, Debug, PartialEq)]
enum VirtioBlkReqType {
    In = VIRTIO_BLK_T_IN,
    Out = VIRTIO_BLK_T_OUT,
}

#[repr(C)]
#[derive(Clone, Debug)]
struct VirtioBlkReq {
    req_type: VirtioBlkReqType,
    reserved: u32,
    sector: u64,
}

#[repr(C)]
#[derive(Clone, Debug)]
struct VirtioQDesc {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

#[repr(C)]
#[derive(Clone)]
struct VirtioQAvail {
    pub flags: u16,
    pub idx: u16,
    pub ring: [u16; VIRTIO_DESC_NUM],
    pub used_event: u16,
}

#[repr(C)]
#[derive(Clone)]
struct VirtioQUsedElem {
    id: u32,
    len: u32,
}

#[repr(C)]
#[derive(Clone)]
struct VirtioQUsed {
    flags: u16,
    idx: u16,
    ring: [VirtioQUsedElem; VIRTIO_DESC_NUM],
    avail_event: u16,
}

pub struct BlockDevice {
    pub storage: Vec<u8>,
    size_in_blocks: usize,
}

const SECTOR_SIZE: usize = 512;
const BLOCK_SIZE: usize = 1024;

impl BlockDevice {
    pub fn new(path: &str) -> std::io::Result<Self> {
        // Get file size
        let metadata = std::fs::metadata(&path)?;
        let file_size = metadata.len() as usize;

        // Calculate number of blocks (rounding up)
        let size_in_blocks = (file_size + SECTOR_SIZE - 1) / SECTOR_SIZE;

        // Create storage with exact size
        let mut storage = vec![0; size_in_blocks * SECTOR_SIZE];

        // Read the image file
        let mut file = File::open(path)?;
        file.read_exact(&mut storage[..file_size])?;

        Ok(BlockDevice {
            storage,
            size_in_blocks,
        })
    }

    pub fn read_block(&self, block_num: usize, len: usize) -> &[u8] {
        if block_num >= self.size_in_blocks {
            panic!("Read outside of block device");
        }
        let start = block_num * SECTOR_SIZE;
        &self.storage[start..start + len]
    }

    pub fn write_block(&mut self, block_num: usize, len: usize, data: &[u8]) {
        if block_num >= self.size_in_blocks {
            panic!("Block number exceeds device size");
        }
        //if data.len() != SECTOR_SIZE {
        //    panic!("Data size must match block size");
        //}

        let start = block_num * SECTOR_SIZE;
        self.storage[start..start + len].copy_from_slice(data);
    }

    pub fn write_to_file(&self, path: &str) -> std::io::Result<()> {
        let mut file = File::create(path)?;

        for i in 0..self.size_in_blocks {
            let block = self.read_block(i, SECTOR_SIZE);
            file.write_all(&block)?;
        }

        file.flush()?;
        Ok(())
    }
}

fn read_virtio_queue_avail_addr(cpu: &mut Cpu) -> u64 {
    let virtio = &mut cpu.peripherals.as_mut().unwrap().virtio;
    let virtio_avail_addr_low = virtio
        .read_mem_u32(VIRTIO_0_ADDR + VIRTIO_MMIO_QUEUE_AVAIL_LOW as u64)
        .unwrap();
    let virtio_avail_addr_high = virtio
        .read_mem_u32(VIRTIO_0_ADDR + VIRTIO_MMIO_QUEUE_AVAIL_HIGH as u64)
        .unwrap();
    (virtio_avail_addr_low as u64) | ((virtio_avail_addr_high as u64) << 32)
}
fn read_virtio_queue_desc_addr(cpu: &mut Cpu) -> u64 {
    let virtio = &mut cpu.peripherals.as_mut().unwrap().virtio;
    let virtio_desc_addr_low = virtio
        .read_mem_u32(VIRTIO_0_ADDR + VIRTIO_MMIO_QUEUE_DESC_LOW as u64)
        .unwrap();
    let virtio_desc_addr_high = virtio
        .read_mem_u32(VIRTIO_0_ADDR + VIRTIO_MMIO_QUEUE_DESC_HIGH as u64)
        .unwrap();
    (virtio_desc_addr_low as u64) | ((virtio_desc_addr_high as u64) << 32)
}

fn read_virtio_queue_used_addr(cpu: &mut Cpu) -> u64 {
    let virtio = &mut cpu.peripherals.as_mut().unwrap().virtio;
    let virtio_used_addr_low = virtio
        .read_mem_u32(VIRTIO_0_ADDR + VIRTIO_MMIO_QUEUE_USED_LOW as u64)
        .unwrap();
    let virtio_used_addr_high = virtio
        .read_mem_u32(VIRTIO_0_ADDR + VIRTIO_MMIO_QUEUE_USED_HIGH as u64)
        .unwrap();
    (virtio_used_addr_low as u64) | ((virtio_used_addr_high as u64) << 32)
}

fn read_mem_virtio_avail(cpu: &mut Cpu) -> VirtioQAvail {
    let virtio_avail_addr = read_virtio_queue_avail_addr(cpu);
    let virtio = &mut cpu.peripherals.as_mut().unwrap().virtio;
    let queue_size = size_of::<VirtioQAvail>();
    let mut buf = vec![0u8; queue_size];
    virtio.read_buf(virtio_avail_addr, &mut buf);
    unsafe { (*(buf.as_ptr() as *const VirtioQAvail)).clone() }
}

fn read_mem_virtio_desc(cpu: &mut Cpu, desc_idx: u16) -> VirtioQDesc {
    let virtio_desc_addr = read_virtio_queue_desc_addr(cpu);
    let virtio = &mut cpu.peripherals.as_mut().unwrap().virtio;
    let desc_size = size_of::<VirtioQDesc>();
    let mut buf = vec![0u8; desc_size];
    virtio.read_buf(
        virtio_desc_addr + (desc_idx as u64) * (desc_size as u64),
        &mut buf,
    );
    unsafe { (*(buf.as_ptr() as *const VirtioQDesc)).clone() }
}

fn read_mem_virtio_used(cpu: &mut Cpu) -> VirtioQUsed {
    let virtio_used_addr = read_virtio_queue_used_addr(cpu);
    let virtio = &mut cpu.peripherals.as_mut().unwrap().virtio;
    let queue_size = size_of::<VirtioQUsed>();
    let mut buf = vec![0u8; queue_size];
    virtio.read_buf(virtio_used_addr, &mut buf);
    unsafe { (*(buf.as_ptr() as *const VirtioQUsed)).clone() }
}

fn write_mem_virtio_used(cpu: &mut Cpu, used: &VirtioQUsed) {
    let virtio_used_addr = read_virtio_queue_used_addr(cpu);
    let virtio = &mut cpu.peripherals.as_mut().unwrap().virtio;
    let queue_size = size_of::<VirtioQUsed>();
    let buf = unsafe {
        std::slice::from_raw_parts((used as *const VirtioQUsed) as *const u8, queue_size)
    };
    virtio.write_buf(virtio_used_addr, buf);
}

fn read_mem_virtio_blk_req(cpu: &mut Cpu, addr: u64) -> VirtioBlkReq {
    let virtio = &mut cpu.peripherals.as_mut().unwrap().virtio;
    let req_size = size_of::<VirtioBlkReq>();
    let mut buf = vec![0u8; req_size];
    virtio.read_buf(addr, &mut buf);
    unsafe { (*(buf.as_ptr() as *const VirtioBlkReq)).clone() }
}

pub fn process_queue(cpu: &mut Cpu) {
    let virtio = &mut cpu.peripherals.as_mut().unwrap().virtio;
    //println!("process_queue");

    //print_desc_table(cpu);

    let virtio_avail = read_mem_virtio_avail(cpu);
    let last_avail_idx = virtio_avail.idx.wrapping_sub(1);
    let desc_idx = virtio_avail.ring[last_avail_idx as usize % VIRTIO_DESC_NUM];

    let req_desc = read_mem_virtio_desc(cpu, desc_idx);
    let req = read_mem_virtio_blk_req(cpu, req_desc.addr);
    //println!("REQ: {:?}", req);
    assert_eq!(req_desc.flags, VRING_DESC_F_NEXT);

    let data_desc = read_mem_virtio_desc(cpu, req_desc.next);
    assert_ne!(data_desc.flags & VRING_DESC_F_NEXT, 0);
    let write = req.req_type == VirtioBlkReqType::In;

    let status_desc = read_mem_virtio_desc(cpu, data_desc.next);
    assert_eq!(status_desc.len, 1);
    assert_eq!(status_desc.flags, VRING_DESC_F_WRITE);

    //println!("\nvirtio requested write: {}\n\n", write);

    match req.req_type {
        VirtioBlkReqType::In => {
            // read data
            let mut device = cpu.block_device.take().expect("No block device");
            let data = device.read_block(req.sector as usize, data_desc.len as usize);
            cpu.write_buf(data_desc.addr, data).unwrap();
            cpu.block_device = Some(device);
        }
        VirtioBlkReqType::Out => {
            // write data
            let mut buf = vec![0u8; BLOCK_SIZE];
            cpu.read_buf(data_desc.addr, buf.as_mut_slice()).unwrap();
            if let Some(ref mut device) = &mut cpu.block_device {
                device.write_block(req.sector as usize, data_desc.len as usize, &buf);
            } else {
                panic!("No block device");
            }
        }
    };

    let current_status = cpu.read_mem_u8(status_desc.addr).unwrap();
    //println!("current status: {}", current_status);
    cpu.write_mem_u8(status_desc.addr, VIRTIO_BLK_S_OK).unwrap();

    let mut virtio_used = read_mem_virtio_used(cpu);
    let used_idx = virtio_used.idx as usize;
    virtio_used.ring[used_idx % VIRTIO_DESC_NUM] = VirtioQUsedElem {
        id: desc_idx as u32,
        len: 0,
    };
    virtio_used.idx += 1;
    write_mem_virtio_used(cpu, &virtio_used);

    plic_trigger_irq(cpu, VIRTIO0_IRQ);
}

fn print_desc_table(cpu: &mut Cpu) {
    for i in 0..VIRTIO_DESC_NUM {
        let desc = read_mem_virtio_desc(cpu, i as u16);
        println!("DESC{}: {:?}", i, desc);
    }
}

pub fn init_virtio(cpu: &mut Cpu) {
    let virtio = &mut cpu.peripherals.as_mut().unwrap().virtio;

    virtio
        .write_mem_u32(VIRTIO_0_ADDR + VIRTIO_MMIO_DEVICE_ID as u64, 2)
        .unwrap();
    virtio
        .write_mem_u32(VIRTIO_0_ADDR + VIRTIO_MMIO_VENDOR_ID as u64, 0x554d4551)
        .unwrap();
    virtio
        .write_mem_u32(VIRTIO_0_ADDR + VIRTIO_MMIO_MAGIC_VALUE as u64, 0x74726976)
        .unwrap();
    virtio
        .write_mem_u32(VIRTIO_0_ADDR + VIRTIO_MMIO_VERSION as u64, 2)
        .unwrap();
    virtio
        .write_mem_u32(
            VIRTIO_0_ADDR + VIRTIO_MMIO_QUEUE_NUM_MAX as u64,
            VIRTIO_DESC_NUM as u32,
        )
        .unwrap();
}
