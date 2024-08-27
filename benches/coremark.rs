use criterion::{criterion_group, criterion_main, Criterion};

use risc_sim::{
    cpu::{cpu_core::Cpu, memory::vec_memory::VecMemory},
    elf::elf_loader::decode_file,
    system::passthrough_kernel::PassthroughKernel,
};

struct CoreMarkBenchmark {
    cpu: Cpu,
}

impl CoreMarkBenchmark {
    fn new() -> Self {
        let mut kernel = PassthroughKernel::default();
        kernel.set_print_stdout(false);
        let mut cpu = Cpu::new(VecMemory::new(), kernel);
        let program = decode_file("tests/coremark");
        cpu.load_program_from_elf(program).unwrap();
        CoreMarkBenchmark { cpu }
    }

    fn run_benchmark(&mut self) -> f64 {
        const COUNT_INTERVAL: u64 = 5000000;
        loop {
            #[cfg(feature = "maxperf")]
            {
                let mut finished = false;
                for _ in 0..COUNT_INTERVAL {
                    match self.cpu.run_cycle_uncheked() {
                        Ok(_) => {
                            // println!("Cycle: {}", count);
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
        self.extract_coremark_score().unwrap()
    }

    fn extract_coremark_score(&mut self) -> Option<f64> {
        let stdout: String = self.cpu.kernel.read_and_clear_stdout_buffer();
        println!("{}", stdout);
        for line in stdout.lines() {
            if line.starts_with("Iterations/Sec") {
                if let Some(score_str) = line.split(":").nth(1) {
                    if let Ok(score) = score_str.trim().parse::<f64>() {
                        return Some(score);
                    }
                }
            }
        }
        None
    }
}

fn benchmark_coremark(c: &mut Criterion) {
    let mut benchmark = CoreMarkBenchmark::new();

    let mut group = c.benchmark_group("coremark");

    group.bench_function("coremark_score", |b| b.iter(|| benchmark.run_benchmark()));

    group.finish();
}

criterion_group!(benches, benchmark_coremark);
criterion_main!(benches);
