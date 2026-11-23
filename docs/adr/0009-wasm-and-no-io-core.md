# ADR-0009: No-I/O core and WebAssembly

Status: Proposed

## Decision
`mkpdf-core` does no I/O, spawns no threads and holds no global state. Inputs and outputs are bytes in memory. File access belongs to the CLI and to callers. This keeps the core buildable for `wasm32` and easy to test.

## Consequences
Large documents are held in memory unless a streaming sink is added later. A `wasm32` build is not part of CI yet, and must be added before WebAssembly is claimed as supported (OQ-09).
