# Rust engineering rules

1. Stable Rust only. The toolchain is set in `rust-toolchain.toml`. Nightly is allowed only for tools run in CI (Miri, fuzzing), never for building the product.
2. Safe Rust by default. `unsafe_code` is denied for the workspace. `mkpdf-core` and `mkpdf-cli` use `#![forbid(unsafe_code)]`.
3. Unsafe code is allowed only in `mkpdf-ffi`, and each `unsafe` block carries a `// SAFETY:` comment that names the invariant it relies on. Each new unsafe function needs a second reviewer, and its contract is stated in `# Safety` docs.
4. No `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `dbg!` or unchecked indexing in non-test code. Return `Result`. Tests are exempt (see `clippy.toml`).
5. No panic may cross an FFI boundary. Every exported function goes through the `guard` helper. `extern "C"` functions never unwind.
6. Arithmetic on sizes, offsets or counts that come from input uses checked or saturating operations. Release builds keep `overflow-checks = true`.
7. Errors: one `Error` type with a stable `ErrorKind`. Messages are for humans and are not a contract.
8. Gates before merge: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo deny check`.
9. Documentation: every public item has a doc comment. Doc examples compile.
10. Dependencies: only through the process in `rules/licensing.md`.
11. Keep functions small and tested. Comments explain why, not what.
