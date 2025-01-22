RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data -Clink-arg=-lgcov" cargo build --target=x86_64-unknown-linux-gnu --release --features maxperf
