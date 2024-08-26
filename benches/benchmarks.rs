use std::time::Duration;

use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion,
    PlotConfiguration,
};
use risc_sim::cpu::{
    cpu_core::Cpu,
    memory::{
        btree_memory::BTreeMemory, hashmap_memory::FxHashMemory, memory_core::Memory,
        vec_binsearch_memory::VecBsearchMemory, vec_memory::VecMemory,
    },
};

// Calculates n-th fibbonacci number and stores it in x5
const FIB_PROGRAM_BIN: &[u32] = &[
    0x00100093, 0x00100113, 0x00002183, // lw x3, x0 - load n from memory
    0x00000213, 0x00010293, 0x00208133, 0x00028093, 0x00120213, 0xfe3248e3, // blt x4, x3, -16
    0xfcdff06f,
];

fn fibbonaci_program(n: u32) {
    let mut cpu = Cpu::default();

    cpu.load_program_from_opcodes(FIB_PROGRAM_BIN.to_vec(), 0)
        .unwrap();

    cpu.write_mem_u32(0, n).unwrap();

    while cpu.run_cycle().is_ok() {}
}

fn read_write_randon_mem(locations: u32, mut mem: impl Memory) {
    const RW_CYCLES: usize = 1;
    const BUF_SIZE: usize = 32;
    const BUF: [u32; BUF_SIZE] = [0; BUF_SIZE];

    for _ in 0..RW_CYCLES {
        for (j, data) in BUF.iter().enumerate().take(BUF_SIZE) {
            for i in 0..locations {
                let addr = (u32::MAX / locations) * i;
                mem.write_mem_u32(addr + (j * 4) as u32, *data).unwrap();
                mem.read_mem_u32(addr + (j * 4) as u32).unwrap();
            }
        }
    }
}

fn bench_fibbonacci(c: &mut Criterion) {
    c.bench_function("fibbonacci", |b| b.iter(|| fibbonaci_program(20)));
}

fn bench_mem_read_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("Memory");

    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    group.plot_config(plot_config);

    group.warm_up_time(Duration::from_millis(100));
    group.measurement_time(Duration::from_millis(500));

    let spans = vec![1, 2, 3, 5, 8, 12, 20, 40, 70, 90, 100];

    for &span in &spans {
        group.bench_with_input(BenchmarkId::new("FxHashMemory", span), &span, |b, &s| {
            b.iter(|| read_write_randon_mem(black_box(s), FxHashMemory::new()))
        });

        group.bench_with_input(BenchmarkId::new("VecMemory", span), &span, |b, &s| {
            b.iter(|| read_write_randon_mem(black_box(s), VecMemory::new()))
        });

        group.bench_with_input(
            BenchmarkId::new("VecBsearchMemory", span),
            &span,
            |b, &s| b.iter(|| read_write_randon_mem(black_box(s), VecBsearchMemory::new())),
        );

        group.bench_with_input(BenchmarkId::new("BTreeMemory", span), &span, |b, &s| {
            b.iter(|| read_write_randon_mem(black_box(s), BTreeMemory::new()))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_mem_read_write, bench_fibbonacci);
criterion_main!(benches);
