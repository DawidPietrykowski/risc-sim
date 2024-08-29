## Benchmarking

##### Coremark
To compile coremark for use with the bench harness you need to installl the riscv-gcc toolchain and run the following make command in the coremark repository.

```make compile PORT_DIR=simple ITERATIONS=500 CC=riscv32-unknown-elf-gcc LD=riscv32-unknown-elf-ld AS=riscv32-unknown-elf-as XCFLAGS="-march=rv32im -mabi=ilp32"```