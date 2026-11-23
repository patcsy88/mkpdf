# ADR-0002: Cargo workspace and the unsafe boundary

Status: Proposed

## Context
The core must be safe and portable. The C ABI needs raw pointers, so some `unsafe` is unavoidable.

## Decision
A Cargo workspace with three crates now: `mkpdf-core`, `mkpdf-ffi`, `mkpdf-cli`. `unsafe_code` is denied for the workspace. `mkpdf-core` and `mkpdf-cli` use `forbid`. Only `mkpdf-ffi` opts in, and each block has a `SAFETY` comment. Further crates are added only when a second consumer needs them.

## Alternatives
- One crate with feature flags: it cannot enforce the boundary.
- Many small crates now: too much structure before the design is known.

## Consequences
Unsafe code is easy to find and review. The FFI crate needs Miri and a Security Engineer review.
