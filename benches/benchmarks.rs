use std::u32;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use risc_sim::{
    asm::assembler::ProgramFile,
    cpu::{
        cpu_core::Cpu,
        memory::{hashmap_memory::FxHashMemory, memory_core::Memory, vec_memory::VecMemory},
    },
};

// Calculates n-th fibbonacci number and stores it in x5
const FIB_PROGRAM_BIN: &[u32] = &[
    0x00100093, 0x00100113, 0x00002183, // lw x3, x0 - load n from memory
    0x00000213, 0x00010293, 0x00208133, 0x00028093, 0x00120213, 0xfe3248e3, // blt x4, x3, -16
    0xfcdff06f,
];

fn fibbonaci_program(n: u32) {
    let mut cpu = Cpu::new();

    let mut memory = VecMemory::new();
    for (id, val) in FIB_PROGRAM_BIN.iter().enumerate() {
        memory.write_mem_u32(4u32 * (id as u32), *val).unwrap();
    }

    cpu.load_program(ProgramFile {
        entry_point: 0,
        memory,
        program_memory_offset: 0,
        lines: vec![],
        program_size: (FIB_PROGRAM_BIN.len() * 4) as u32,
    });

    cpu.write_mem_u32(0, n).unwrap();

    while cpu.run_cycle().is_ok() {
        // println!("{}", cpu);
    }
}

fn read_write_randon_mem(locations: u32, mut mem: impl Memory) {
    for i in 0..locations {
        let addr = (u32::MAX / locations) * i;
        mem.write_mem_u32(addr, i).unwrap();
        mem.read_mem_u32(addr).unwrap();
    }
}

fn bench_fibbonacci(c: &mut Criterion) {
    c.bench_function("fibbonacci", |b| b.iter(|| fibbonaci_program(20)));
}

fn bench_mem_read_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("Memory");
    let span = 1000;

    group.bench_function(BenchmarkId::new("FxHashMemory", span), |b| {
        b.iter(|| read_write_randon_mem(black_box(span), FxHashMemory::new()))
    });

    group.bench_function(BenchmarkId::new("VecMemory", span), |b| {
        b.iter(|| read_write_randon_mem(black_box(span), VecMemory::new()))
    });

    group.finish();
}

criterion_group!(benches, bench_mem_read_write, bench_fibbonacci);
criterion_main!(benches);
