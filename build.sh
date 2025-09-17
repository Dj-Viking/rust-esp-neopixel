#! /usr/bin/bash
cargo build

doas espflash flash ./target/riscv32imac-unknown-none-elf/debug/rust-esp-shit

# doas espflash monitor
