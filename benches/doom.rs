use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::{
    fmt::Display,
    time::{Duration, Instant},
};

use risc_sim::{
    cpu::{cpu_core::Cpu, memory::vec_memory::VecMemory},
    elf::elf_loader::decode_file,
    system::passthrough_kernel::PassthroughKernel,
};

#[derive(Clone, Copy, Debug)]
enum BenchmarkType {
    Regular,
    Skips,
}

impl Display for BenchmarkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BenchmarkType::Regular => write!(f, "regular"),
            BenchmarkType::Skips => write!(f, "skips"),
        }
    }
}

struct CpuBenchmark {
    cpu: Cpu,
}

impl CpuBenchmark {
    fn new() -> Self {
        let mut kernel = PassthroughKernel::default();
        kernel.set_print_stdout(false);
        let mut cpu = Cpu::new(VecMemory::new(), kernel);
        let program = decode_file("doomgeneric"); // TODO: set with env var (?)
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

    fn run_benchmark_skips(&mut self, duration: Duration) -> f64 {
        const COUNT_INTERVAL: u64 = 5000000;

        let start = Instant::now();

        let mut count = 0;
        loop {
            count += COUNT_INTERVAL;
            if start.elapsed() >= duration {
                // no need to check evey cycle
                break;
            }
            for _ in 0..COUNT_INTERVAL {
                self.cpu.run_cycle_uncheked();
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

    for benchmark_type in [BenchmarkType::Regular, BenchmarkType::Skips].iter() {
        group.bench_with_input(
            BenchmarkId::new("mhz", benchmark_type),
            benchmark_type,
            |b, &t| {
                b.iter_custom(|iters| {
                    let mut total_mhz = 0.0;
                    for _ in 0..iters {
                        total_mhz += match t {
                            BenchmarkType::Regular => {
                                benchmark.run_benchmark(Duration::from_secs(1))
                            }
                            BenchmarkType::Skips => {
                                benchmark.run_benchmark_skips(Duration::from_secs(1))
                            }
                        };
                    }
                    Duration::from_secs_f64(1.0 / total_mhz)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_doom_intro);
criterion_main!(benches);
