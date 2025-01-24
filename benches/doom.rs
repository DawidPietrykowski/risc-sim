use criterion::{criterion_group, criterion_main, Criterion};
use std::time::{Duration, Instant};

use risc_sim::{
    cpu::cpu_core::{Cpu, CpuMode},
    elf::elf_loader::{decode_file, WordSize},
};

struct CpuBenchmark {
    cpu: Cpu,
}

impl CpuBenchmark {
    fn new() -> Self {
        let program = decode_file("doomgeneric"); // TODO: set with env var (?)
        let mode = match program.header.word_size {
            WordSize::W32 => CpuMode::RV32,
            WordSize::W64 => CpuMode::RV64,
        };
        let mut cpu = Cpu::new_userspace(mode);
        cpu.load_program_from_elf(program).unwrap();
        CpuBenchmark { cpu }
    }

    fn run_benchmark(&mut self, duration: Duration) -> f64 {
        let start = Instant::now();

        let mut count = 0;
        const INTERVAL: u64 = 5000;
        loop {
            count += INTERVAL;
            self.cpu.run_cycles(INTERVAL).unwrap();
            if start.elapsed() >= duration {
                break;
            }
        }

        let elapsed = start.elapsed();

        count as f64 / elapsed.as_secs_f64() / 1_000_000.0
    }
}

fn benchmark_doom_intro(_c: &mut Criterion) {
    let mut benchmark = CpuBenchmark::new();

    let mut custom_config = Criterion::default().configure_from_args();
    custom_config = custom_config
        .sample_size(10)
        .measurement_time(Duration::from_secs(20));

    let mut group = custom_config.benchmark_group("doom_intro");

    group.bench_function("benchmark_doom_intro", |b| {
        b.iter_custom(|iters| {
            let mut total_mhz = 0.0;
            for _ in 0..iters {
                total_mhz += benchmark.run_benchmark(Duration::from_secs(1));
            }
            Duration::from_secs_f64(1.0 / total_mhz)
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_doom_intro);
criterion_main!(benches);
