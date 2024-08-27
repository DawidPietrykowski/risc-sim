use std::time::Duration;

use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion,
    PlotConfiguration,
};
use risc_sim::cpu::memory::{
    btree_memory::BTreeMemory, hashmap_memory::FxHashMemory, memory_core::Memory,
    vec_binsearch_memory::VecBsearchMemory, vec_memory::VecMemory,
};

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

criterion_group!(benches, bench_mem_read_write);
criterion_main!(benches);
