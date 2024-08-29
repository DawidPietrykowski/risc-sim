use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};

use risc_sim::{
    cpu::{
        cpu_core::Cpu,
        memory::{
            btree_memory::BTreeMemory, hashmap_memory::FxHashMemory, memory_core::Memory,
            vec_binsearch_memory::VecBsearchMemory, vec_memory::VecMemory,
            vec_u8_memory::VecU8Memory,
        },
    },
    elf::elf_loader::decode_file,
    system::passthrough_kernel::PassthroughKernel,
};

fn run_benchmark_with_mem<M>(mem: M)
where
    M: Memory + 'static,
{
    let mut kernel = PassthroughKernel::default();
    kernel.set_print_stdout(false);
    let mut cpu = Cpu::new(mem, kernel);
    let program = decode_file("tests/coremark.elf");
    cpu.load_program_from_elf(program).unwrap();
    const COUNT_INTERVAL: u64 = 50000;
    loop {
        let mut finished = false;
        for _ in 0..COUNT_INTERVAL {
            match cpu.run_cycle_uncheked() {
                Ok(_) => {
                    continue;
                }
                Err(_e) => {
                    finished = true;
                    break;
                }
            };
        }
        if finished {
            break;
        }
    }
}

fn bench_mem_read_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("Memory");

    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_millis(2000));
    group.sample_size(100);

    group.bench_function("VecMemory", |b| {
        b.iter(|| run_benchmark_with_mem(mem))
    });

    group.bench_function("TableMemory", |b| {
        b.iter(|| run_benchmark_with_mem(TableMemory::new()))
    });

    // group.bench_function("VecMemoryCache", |b| {
    //     b.iter(|| run_benchmark_with_mem(VecMemoryCache::new()))
    });

    group.bench_function("VecBsearchMemory", |b| {
        b.iter(|| run_benchmark_with_mem(VecBsearchMemory::new()))
    });

    group.bench_function("VecU8Memory", |b| {
        b.iter(|| run_benchmark_with_mem(VecU8Memory::new()))
    });

    group.bench_function("FxHashMemory", |b| {
        b.iter(|| run_benchmark_with_mem(FxHashMemory::new()))
    });

    group.bench_function("BTreeMemory", |b| {
        b.iter(|| run_benchmark_with_mem(BTreeMemory::new()))
    });

    group.finish();
}

criterion_group!(benches, bench_mem_read_write);
criterion_main!(benches);
