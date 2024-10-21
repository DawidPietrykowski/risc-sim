# RISC-V Emulator

A highly functional RISC-V emulator capable of running complex applications like the original DOOM at playable framerates.

## Features

- 32-bit and 64-bit support
- Simulated MMU with virtual address translation and paging (in progress)
- ELF parsing and memory mapping for RISC-V binaries
- Instruction set support:
  - Base integer instructions (I)
  - Integer multiplication and division (M)
  - Floating-point operations (F) and double-precision (D) (in progress)
  - Atomic operations (A)
- System call handling (fstat, write, brk, gettime, etc.)
- Host system call passthrough
- Privileged architecture support (in progress)
- Extensive automated testing suite
- Benchmarking tools

## Current Usecases

- Run C programs compiled for RISC-V
- Run complex applications (e.g., DOOM at ~20 FPS)
- Write custom extensions and modifications based on existing library implementations

## Upcoming Features

- Complete G extension support (F, D)
- Additional privileged modes (Supervisor, Machine)
- Enhanced debugging capabilities
- Peripheral device simulation (GPIO, UART)

## Build and Run

```
cargo run
``` 

## Testing

The project includes a comprehensive test suite covering:
- Individual instructions
- Memory operations
- Instruction sets (e.g., Fibonacci sequence calculation)
- Full C program execution and output comparison with known-good results

To run the tests:

```
cargo test
```

## Benchmarks

The project includes automated benchmarking tools. To run benchmarks and generate performance graphs:

```
cargo bench
```

##### Coremark
To compile coremark for use with the bench harness you need to installl the riscv-gcc toolchain and run the following make command in the coremark repository.

```make compile PORT_DIR=simple ITERATIONS=500 CC=riscv32-unknown-elf-gcc LD=riscv32-unknown-elf-ld AS=riscv32-unknown-elf-as XCFLAGS="-march=rv32im -mabi=ilp32"```

## License

This project is currently licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

We reserve the right to change the license in future versions. All contributions to the current version will remain under the MIT License.

For any licensing questions or to discuss commercial use, please contact contact@dawidpietrykowski.com