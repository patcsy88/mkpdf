# ADR-0009: No-I/O core and WebAssembly

Status: Proposed

## Decision
`mkpdf-core` does no I/O, spawns no threads and holds no global state. Inputs and outputs are bytes in memory. File access belongs to the CLI and to callers. This keeps the core buildable for `wasm32` and easy to test.

## Consequences
Large documents are held in memory unless a streaming sink is added later. A CI job (`wasm-core`) builds `mkpdf-core` for `wasm32-unknown-unknown` and passed in CI run 45103722429 (updated 2026-11-24, ARCH-F-12; the earlier text said the build was not in CI). The job only compiles. No test runs on wasm, so WebAssembly must not be claimed as supported until one does (OQ-09). Status unchanged: Proposed.
