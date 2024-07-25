#!/bin/bash

# Directory containing the C programs
TEST_DIR="tests"

# Compile and run each C program in the tests directory
for c_file in "$TEST_DIR"/*.c; do
    if [ -f "$c_file" ]; then
        # Get the filename without extension
        filename=$(basename -- "$c_file")
        name_no_ext="${filename%.*}"

        # Compile the C program
        riscv32-unknown-elf-gcc -march=rv32im -mabi=ilp32 -o "$TEST_DIR/$name_no_ext" "$c_file"

        # Run the program and save output to .res file
        qemu-riscv32 "$TEST_DIR/$name_no_ext" > "$TEST_DIR/$name_no_ext.res"

        echo "Processed: $filename"
    fi
done

echo "All tests generated."
