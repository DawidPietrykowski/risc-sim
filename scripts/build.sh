name=$1
riscv32-unknown-elf-gcc -march=rv32im -mabi=ilp32 $name.c -o $name
# riscv32-unknown-elf-gcc -march=rv32im -mabi=ilp32 -mno-shorten-memrefs -O0 -fno-inline -fno-optimize-sibling-calls $name.c -o $name
# riscv32-unknown-elf-objdump -d $name
# riscv64-unknown-elf-as -march=rv32i -mabi=ilp32 $name.s -o $name.o
# riscv64-unknown-elf-ld -m elf32lriscv  $name.o -o $name