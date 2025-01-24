use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};

use risc_sim::{
    cpu::{
        cpu_core::{Cpu, CpuMode, ExecutionMode},
        memory::{
            btree_memory::BTreeMemory, hashmap_memory::FxHashMemory, memory_core::Memory,
            raw_table_memory::RawTableMemory, raw_vec_memory::RawVecMemory,
            table_memory::TableMemory, user_memory::UserMemory,
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
    let mut cpu = Cpu::new(mem, kernel, CpuMode::RV32, None, ExecutionMode::UserSpace);
    let program = decode_file("tests/coremark.elf");
    cpu.load_program_from_elf(program).unwrap();
    const COUNT_INTERVAL: u64 = 50000;
    loop {
        match cpu.run_cycles(COUNT_INTERVAL) {
            Ok(_) => {
                continue;
            }
            Err(_e) => {
                break;
            }
        };
    }
}

fn bench_mem_read_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("Coremark Memory");

    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_millis(2000));
    group.sample_size(10);

    group.bench_function("TableMemory", |b| {
        let mem = TableMemory::new();
        b.iter(|| run_benchmark_with_mem(mem.clone()))
    });

    group.bench_function("RawTableMemory", |b| {
        let mem = RawTableMemory::new();
        b.iter(|| run_benchmark_with_mem(mem.clone()))
    });

    group.bench_function("VecMemory", |b| {
        let mem = VecMemory::new();
        b.iter(|| run_benchmark_with_mem(mem.clone()))
    });

    group.bench_function("VecBsearchMemory", |b| {
        let mem = VecBsearchMemory::new();
        b.iter(|| run_benchmark_with_mem(mem.clone()))
    });

    group.bench_function("VecU8Memory", |b| {
        let mem = VecU8Memory::new();
        b.iter(|| run_benchmark_with_mem(mem.clone()))
    });

    group.bench_function("FxHashMemory", |b| {
        let mem = FxHashMemory::new();
        b.iter(|| run_benchmark_with_mem(mem.clone()))
    });

    group.bench_function("BTreeMemory", |b| {
        let mem = BTreeMemory::new();
        b.iter(|| run_benchmark_with_mem(mem.clone()))
    });

    group.bench_function("RawVecMemory", |b| {
        let mem = RawVecMemory::new();
        b.iter(|| run_benchmark_with_mem(mem.clone()))
    });

    group.bench_function("UserMemory", |b| {
        let mem = UserMemory::new_32();
        b.iter(|| run_benchmark_with_mem(mem.clone()))
    });

    group.finish();
}

criterion_group!(benches, bench_mem_read_write);
criterion_main!(benches);
