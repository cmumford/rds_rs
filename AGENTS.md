# Agent Guide: rds-rs

This file is a high-signal reference for AI agents to avoid repository-specific mistakes and ramp up quickly.

## Environment & Constraints

*   **Strict `no_std`:** The library crate (`src/`) is entirely `#![no_std]`.
    *   No `std::` library access is allowed.
    *   Uses `heapless` for static/fixed-capacity collections (e.g., `heapless::String`).
    *   `thiserror` is imported with `default-features = false`.
*   **Bitfields:** Code uses `modular-bitfield-msb` (MSB-first bit layout), **not** the standard `modular-bitfield` crate. Always import from `modular_bitfield_msb::prelude::*`.
*   **Directories to Ignore:**
    *   Never search, read, or scan the `third_party/` directory (which contains extensive RDS spy logs) or `.jj/` (Jujutsu VCS metadata directory).

## Commands & Workflows

### Verification Pipeline
Always verify changes in this sequence:
1.  **Check:** `make check` (or `cargo check`)
2.  **Lint:** `make clippy` (runs clippy with `--all-targets`)
3.  **Test:** `cargo test` (unit and integration tests)
4.  **Smoke Test:** `make test` (runs decode example on a Swedish spy log file)
5.  **Full Test:** `make test-all` (decodes all spy logs in the directory)

### Fuzzing
*   **Run Fuzzer:** `make fuzz` (executes `cargo fuzz run fuzz_target_1`). Requires a nightly Rust toolchain.

## Key Architecture Nodes
*   **Decoder State:** `src/rds.rs` (`RdsData`) stores the current RDS decoder state.
*   **Decoding Entrypoint:** `src/decoder.rs` contains the core `Decoder` struct and state machine implementing the spec decoding.
*   **Utility & Tables:** `src/text.rs` holds the custom RDS character set translation table (EBU common language mapping to UTF-8).
