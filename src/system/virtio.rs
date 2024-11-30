use crate::{cpu::cpu_core::Cpu, system::plic::plic_trigger_irq};

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

#[repr(C)]
#[derive(Clone)]
struct VirtioBlkReq {
    req_type: u32,
    reserved: u32,
    sector: u64,
}

#[repr(C)]
#[derive(Clone)]
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

#[allow(non_camel_case_types)]
enum VirtioBlkReqType {
    VIRTIO_BLK_T_IN = 0,
    VIRTIO_BLK_T_OUT = 1,
}

fn read_virtio_queue_avail_addr(cpu: &mut Cpu) -> u64 {
    let virtio_avail_addr_low = cpu
        .read_mem_u32(VIRTIO_0_ADDR + VIRTIO_MMIO_QUEUE_AVAIL_LOW as u64)
        .unwrap();
    let virtio_avail_addr_high = cpu
        .read_mem_u32(VIRTIO_0_ADDR + VIRTIO_MMIO_QUEUE_AVAIL_HIGH as u64)
        .unwrap();
    (virtio_avail_addr_low as u64) | ((virtio_avail_addr_high as u64) << 32)
}
fn read_virtio_queue_desc_addr(cpu: &mut Cpu) -> u64 {
    let virtio_desc_addr_low = cpu
        .read_mem_u32(VIRTIO_0_ADDR + VIRTIO_MMIO_QUEUE_DESC_LOW as u64)
        .unwrap();
    let virtio_desc_addr_high = cpu
        .read_mem_u32(VIRTIO_0_ADDR + VIRTIO_MMIO_QUEUE_DESC_HIGH as u64)
        .unwrap();
    (virtio_desc_addr_low as u64) | ((virtio_desc_addr_high as u64) << 32)
}

fn read_virtio_queue_used_addr(cpu: &mut Cpu) -> u64 {
    let virtio_used_addr_low = cpu
        .read_mem_u32(VIRTIO_0_ADDR + VIRTIO_MMIO_QUEUE_USED_LOW as u64)
        .unwrap();
    let virtio_used_addr_high = cpu
        .read_mem_u32(VIRTIO_0_ADDR + VIRTIO_MMIO_QUEUE_USED_HIGH as u64)
        .unwrap();
    (virtio_used_addr_low as u64) | ((virtio_used_addr_high as u64) << 32)
}

fn read_mem_virtio_avail(cpu: &mut Cpu) -> VirtioQAvail {
    let virtio_avail_addr = read_virtio_queue_avail_addr(cpu);
    let queue_size = size_of::<VirtioQAvail>();
    let mut buf = vec![0u8; queue_size];
    cpu.read_buf(virtio_avail_addr, &mut buf);
    unsafe { (*(buf.as_ptr() as *const VirtioQAvail)).clone() }
}

fn read_mem_virtio_desc(cpu: &mut Cpu, desc_idx: u16) -> VirtioQDesc {
    let virtio_desc_addr = read_virtio_queue_desc_addr(cpu);
    let desc_size = size_of::<VirtioQDesc>();
    let mut buf = vec![0u8; desc_size];
    cpu.read_buf(
        virtio_desc_addr + (desc_idx as u64) * (desc_size as u64),
        &mut buf,
    );
    unsafe { (*(buf.as_ptr() as *const VirtioQDesc)).clone() }
}

fn read_mem_virtio_used(cpu: &mut Cpu) -> VirtioQUsed {
    let virtio_used_addr = read_virtio_queue_used_addr(cpu);
    let queue_size = size_of::<VirtioQUsed>();
    let mut buf = vec![0u8; queue_size];
    cpu.read_buf(virtio_used_addr, &mut buf);
    unsafe { (*(buf.as_ptr() as *const VirtioQUsed)).clone() }
}

fn read_mem_virtio_blk_req(cpu: &mut Cpu, addr: u64) -> VirtioBlkReq {
    let req_size = size_of::<VirtioBlkReq>();
    let mut buf = vec![0u8; req_size];
    cpu.read_buf(addr, &mut buf);
    unsafe { (*(buf.as_ptr() as *const VirtioBlkReq)).clone() }
}

pub fn process_queue(cpu: &mut Cpu) {
    println!("process_queue");
    
    let virtio_avail = read_mem_virtio_avail(cpu);
    let last_avail_idx = virtio_avail.idx.wrapping_sub(1);
    let desc_idx = virtio_avail.ring[last_avail_idx as usize % VIRTIO_DESC_NUM];

    let req_desc = read_mem_virtio_desc(cpu, desc_idx);
    let req = read_mem_virtio_blk_req(cpu, req_desc.addr);
    assert_eq!(req_desc.flags, VRING_DESC_F_NEXT);

    let data_desc = read_mem_virtio_desc(cpu, req_desc.next);
    assert_ne!(data_desc.flags & VRING_DESC_F_NEXT, 0);
    let write = (data_desc.flags & VRING_DESC_F_WRITE) != 0;

    let status_desc = read_mem_virtio_desc(cpu, data_desc.next);
    assert_eq!(status_desc.len, 1);
    assert_eq!(status_desc.flags, VRING_DESC_F_WRITE);

    // TODO: Perform read/write and raise interrupt

    plic_trigger_irq(cpu, VIRTIO0_IRQ);
}

pub fn init_virtio(cpu: &mut Cpu) {
    cpu.write_mem_u32(VIRTIO_0_ADDR + VIRTIO_MMIO_DEVICE_ID as u64, 2)
        .unwrap();
    cpu.write_mem_u32(VIRTIO_0_ADDR + VIRTIO_MMIO_VENDOR_ID as u64, 0x554d4551)
        .unwrap();
    cpu.write_mem_u32(VIRTIO_0_ADDR + VIRTIO_MMIO_MAGIC_VALUE as u64, 0x74726976)
        .unwrap();
    cpu.write_mem_u32(VIRTIO_0_ADDR + VIRTIO_MMIO_VERSION as u64, 2)
        .unwrap();
    cpu.write_mem_u32(
        VIRTIO_0_ADDR + VIRTIO_MMIO_QUEUE_NUM_MAX as u64,
        VIRTIO_DESC_NUM as u32,
    )
    .unwrap();
}
