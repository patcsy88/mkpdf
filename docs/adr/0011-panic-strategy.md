# ADR-0011: Panic strategy (`unwind` or `abort`)

Status: Proposed. Resolves open question OQ-12. Not accepted. Only the project owner accepts it, after the Security Engineer has reviewed it (security finding SEC-F-14).

## Context

`mkpdf-ffi` builds a `cdylib` and a `staticlib` (`crates/mkpdf-ffi/Cargo.toml:13`). Every exported function runs inside `guard`, which calls `catch_unwind` (`crates/mkpdf-ffi/src/lib.rs:96-111`) and returns `MKPDF_ERR_INTERNAL` on a panic (ADR-0003, `rules/rust-engineering.md` item 5).

`catch_unwind` only catches anything when the panic strategy is `unwind`. Today `Cargo.toml` sets no `panic` key (`Cargo.toml:34-38`), so `unwind` applies by default. Nothing pins it, nothing tests that it holds in the shipped artefact, and a downstream that rebuilds the crate with `-C panic=abort` silently turns every `guard` into dead code.

Facts checked while writing this ADR (macOS arm64, local, 2026-11-23, Rust 1.100.0):

- A stand-alone program that calls `catch_unwind` around `panic!` exits 0 with `-C panic=unwind` and exits 134 (SIGABRT) with `-C panic=abort`. With `abort`, `catch_unwind` compiles but never catches.
- `#[cfg(panic = "abort")] compile_error!(..)` works on stable. It fires with `-C panic=abort` and compiles with `unwind`.
- A release build of `mkpdf-ffi` with `--config profile.release.panic="abort"` builds. Sizes, unwind vs abort: `cdylib` 429 616 vs 428 256 bytes, `staticlib` 8 251 736 vs 8 209 792 bytes. This crate has a few hundred lines and no dependencies, so the numbers say little about the size cost once the writer, fonts and compression exist. They are not a basis for a decision.
- Not measured: run-time cost, the size of a realistic build, behaviour on Linux and Windows, behaviour under Miri.

Facts from the language (not run here): since Rust 1.81 an unwinding panic that reaches an `extern "C"` function aborts the process. So with `unwind` and no `guard`, the worst case is the same as `abort`. Stack overflow and out-of-memory (allocation failure) abort in both modes and `catch_unwind` cannot stop them. Cargo ignores the `panic` profile key for the `test` profile, so `cargo test` always unwinds, whatever the manifest says. On `wasm32-unknown-unknown` with stable Rust there is no unwinding, so a panic always aborts.

## Decision (proposed)

Use `panic = "unwind"` for `mkpdf-ffi` and the library artefacts, pinned and enforced:

1. Set `panic = "unwind"` explicitly in `[profile.release]` and `[profile.dev]` in the workspace `Cargo.toml`, with a comment that points to this ADR.
2. Add `#[cfg(all(panic = "abort", not(target_arch = "wasm32")))] compile_error!("mkpdf-ffi requires panic=unwind (ADR-0011)");` to `mkpdf-ffi`, so a downstream `-C panic=abort` build fails instead of silently losing the guard. The `wasm32` exclusion is needed only if the FFI crate is ever built for wasm. `mkpdf-core` is built for wasm and has no guard.
3. Add a CI check that fails when the `panic` setting in the manifest changes. Add a test that drives a panic through a real exported function in the built artefact (test-only hook, behind a feature that is never in a release build). Today `panic_is_contained` tests `guard` only (`lib.rs:352-357`).
4. A caught panic must leave the handle in a defined state. Proposal for the owner and the Security Engineer: a `poisoned` flag set by `guard` on a handle that was in use when the panic happened, after which every call except `*_free` returns `MKPDF_ERR_INTERNAL` (security finding SEC-F-03, option b). With D-08 (generation-checked handles) the flag lives in the handle table entry. Until the writer lands, the weaker rule "validate first, mutate last" is enough, because every mutation today is a single assignment or a `push` after the limit check.
5. Document for SDK authors: `MKPDF_ERR_INTERNAL` means a library bug, the handle that raised it should be freed and not reused, and the process continues. Out-of-memory and stack overflow still abort the host. They are limited by documented limits (`rules/architecture.md` item 6), not by `guard`.
6. `abort` is not supported for the shipped libraries. A host that wants fail-fast behaviour can terminate on `MKPDF_ERR_INTERNAL` itself.

## Alternatives

### A. `panic = "abort"` for the cdylib and staticlib

- For: smaller binaries and slightly faster code (no unwind tables, more inlining). No `UnwindSafe` question and no half-mutated state after a panic. Fail-fast on a bug.
- Against: every panic is a host crash. The host is an in-process Node, Python, .NET, Java or Go application, often a server, so one bad document kills every other request (denial of service). The `guard` and `catch_unwind` code, the `panic_is_contained` test and threat T-01's mitigation would be dead or have to be removed. `MKPDF_ERR_INTERNAL` would never be returned. A crash inside a foreign runtime also produces a poor diagnostic (an abort with no managed stack).
- Also: it does not remove the need for no-panic discipline. It only changes the consequence.
- Rejected, because the failure mode is worse for the stated hosts, and the size gain is not shown to matter.

### B. `unwind`, but only `guard` on some functions

Rejected. Partial coverage cannot be reviewed, and `rules/rust-engineering.md` item 5 asks for all of them.

### C. `abort` in the core and `unwind` in the FFI crate

Not possible per crate. The strategy is chosen by the final artefact's profile, not by individual crates in a workspace.

### D. Leave it undecided and rely on the default

Rejected. This is the current state. It is not tested and not enforced, and a downstream rebuild can break it without notice (SEC-F-14).

## Consequences

- Panic containment (threat T-01) becomes a tested property of the shipped library, not an assumption.
- Binaries are somewhat larger, by an amount to be measured on the real writer before v0.1.0.
- Code in the FFI crate must be written so that a caught panic cannot leave shared state broken (item 4).
- Callbacks into foreign code (a streaming sink) would add foreign unwinding into Rust frames, which `unwind` does not make safe by itself. ADR-0012 does not use callbacks for that reason. A later callback ADR must address it.
- On `wasm32` the guard is moot: a panic aborts the module. WASM bindings must say so.
- Needs a Security Engineer review before acceptance.

## Open

- Whether to use `extern "C-unwind"` anywhere. Proposed: no, never.
- Size and run-time measurements on a realistic build.
