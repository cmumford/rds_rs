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

## Core Principles
- **Think Before Coding:** Always analyze the codebase and state your plan before making any edits.
- **Simplicity First:** Implement the simplest solution that solves the problem. Avoid "clever" code or unrequested abstractions.
- **Surgical Edits:** Only modify the specific lines or functions necessary for the task. Do not refactor unrelated code or fix "style" issues unless asked.
- **Goal-Driven:** Define what "success" looks like before you start. Use the `!bash` tool to verify changes with tests or linting immediately after editing.

## Communication Guidelines
- Be concise and technical.
- If a task is ambiguous, ask for clarification instead of guessing.
- When using `!bash` commands, explain what you are checking (e.g., "Checking if the build still passes...").

## Technical Context
- **Style:** Follow the existing patterns in the codebase. Use @filename to reference existing examples for consistency.
- **Testing:** Run `cargo test` after every significant logic change.

## Error Handling
- If a command fails, analyze the stderr output fully before trying a second time.
- Do not enter a loop of "trying things" to see if they work. If you are stuck, stop and ask the user.

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
*   **RDS Documenttion**: is stored in the docs/ directory.
