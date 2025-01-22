#!/bin/bash
rm -rf /tmp/pgo-data/home
llvm-profdata merge -o "/tmp/merged.profdata" "/tmp/pgo-data/"
RUSTFLAGS="-Cprofile-use=/tmp/merged.profdata" cargo build --release --features maxperf
