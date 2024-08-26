use criterion::{criterion_group, criterion_main, Criterion};
use std::time::{Duration, Instant};

use risc_sim::{asm::assembler::decode_file, cpu::cpu_core::Cpu};

struct CpuBenchmark {
    cpu: Cpu,
}

impl CpuBenchmark {
    fn new() -> Self {
        let mut cpu = Cpu::default();
        let program = decode_file("../doomgeneric/doomgeneric/doomgeneric"); // TODO: set with env var (?)
        cpu.load_program_from_elf(program).unwrap();
        CpuBenchmark { cpu }
    }

    fn run_benchmark(&mut self, duration: Duration) -> f64 {
        let start = Instant::now();

        let mut count = 0;
        loop {
            count += 1;
            if count % 50000 == 0 && start.elapsed() >= duration {
                // no need to check evey cycle
                break;
            }
            self.cpu.run_cycle_uncheked();
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

    custom_config.bench_function("doom_intro_mhz", |b| {
        b.iter_custom(|iters| {
            let mut total_mhz = 0.0;
            for _ in 0..iters {
                total_mhz += benchmark.run_benchmark(Duration::from_secs(1));
            }
            Duration::from_secs_f64(1.0 / total_mhz)
        })
    });
}

criterion_group!(benches, benchmark_doom_intro);
criterion_main!(benches);
