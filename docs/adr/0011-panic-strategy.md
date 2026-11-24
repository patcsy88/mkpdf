# ADR-0011: Panic strategy (`unwind` or `abort`)

Status: Proposed. Resolves open question OQ-12. Not accepted. Only the project owner accepts it, after the Security Engineer has reviewed this amended text (security finding SEC-F-14). Related: ADR-0013 (the poison rule), SEC-ADR-06 to SEC-ADR-09, SEC-ADR-13, RISK-F-P2-02, RISK-F-P2-08, RISK-F-P2-09.

## Changes since first draft

| Change | Reason |
|---|---|
| The `compile_error!` condition is now positive: `not(panic = "unwind")`. The `wasm32` exclusion is dropped | SEC-ADR-06 |
| The workspace profile is described as pinning only our own artefacts. The `compile_error!` is the control that reaches a downstream build. The manifest check is secondary | SEC-ADR-06 |
| The panic test must run against the release `cdylib` through a real export, from a separate program. `cargo test` always unwinds and cannot show this | SEC-ADR-06 |
| The test hook uses `--cfg mkpdf_test_hooks`, not a Cargo feature. A symbol check on the shipped library is added | SEC-ADR-07, RISK-F-P2-09 |
| New list of failures that `guard` cannot stop, including `panic_nounwind` aborts and a panic during unwinding | SEC-ADR-08, RISK-F-P2-08 |
| New item 7: scope of `guard`. The payload drop and the error-state work move inside it. `Arc` and lock-guard drops run inside it | SEC-ADR-08, SEC-ADR-03 |
| Poison rule linked to ADR-0013 section 4, with `clear_poison` forbidden | SEC-ADR-03 |
| Sequencing rule: no new mutating export before the poison rule exists | SEC-ADR-09 |
| Claim of a "tested property" reworded as a future property | SEC-ADR-13 |
| CI evidence added: CI run 45103722429 is green. It has no panic-strategy check | RISK-F-P2-02 |

### Findings not accepted, or accepted only in part

| Finding | Disposition | Reason |
|---|---|---|
| SEC-ADR-08 item 1: add a "`Drop` must not panic" rule to `rules/rust-engineering.md` | Proposed to the owner, not made | Only the owner approves changes to the rules files. Item 6 below states the requirement for the FFI crate meanwhile |
| SEC-ADR-08 item 5: CI grep or clippy rule against `extern "C-unwind"` | Proposed, not written | CI and lint config belong to devops. The ADR states the rule and the check |
| SEC-ADR-09: sequencing rule | Recorded here as a proposal. The Orchestrator owns the ordering | I cannot order the backlog. D-08 already puts FFI-003 before FFI-001, and this extends it to every mutating export |
| SEC-ADR-06: "item 1 pins only our own artefacts" | Accepted, and I did not test it | The Cargo behaviour (profiles apply only at the build root) is from knowledge, not run here. The ADR says the `compile_error!` is the control, so the decision does not depend on it |

## Context

`mkpdf-ffi` builds a `cdylib` and a `staticlib` (`crates/mkpdf-ffi/Cargo.toml:13`). Every exported function runs inside `guard`, which calls `catch_unwind` and returns `MKPDF_ERR_INTERNAL` on a panic (ADR-0003, `rules/rust-engineering.md` item 5).

`catch_unwind` only catches anything when the panic strategy is `unwind`. `Cargo.toml` sets no `panic` key, so `unwind` applies by default. Nothing pins it, nothing tests that it holds in the shipped artefact, and a downstream that rebuilds the crate with `-C panic=abort` silently turns every `guard` into dead code.

Facts checked while writing the first draft (macOS arm64, local, 2026-11-23, Rust 1.100.0). Not re-run by the Security Engineer:

- A stand-alone program that calls `catch_unwind` around `panic!` exits 0 with `-C panic=unwind` and exits 134 (SIGABRT) with `-C panic=abort`. With `abort`, `catch_unwind` compiles but never catches.
- `#[cfg(panic = "abort")] compile_error!(..)` works on stable.
- **Checked again for this amendment** (same machine, Rust 1.100.0): `#[cfg(not(panic = "unwind"))] compile_error!(..)` compiles and runs with the default and fails to compile with `-C panic=abort`. So the positive form works on stable.
- A release build of `mkpdf-ffi` with `--config profile.release.panic="abort"` builds. Sizes, unwind vs abort: `cdylib` 429 616 vs 428 256 bytes, `staticlib` 8 251 736 vs 8 209 792 bytes. The crate has a few hundred lines and no dependencies, so the numbers say little about a realistic build.
- Not measured: run-time cost, the size of a realistic build, behaviour on Linux and Windows.

CI evidence (RISK-F-P2-02): CI run 45103722429 (2026-11-23, commit ccc73d5) is green on all 10 jobs, including `miri-ffi` (nightly `rustc 1.102.0-nightly`, 12 tests passed) and `msrv` (`rustc 1.85.0`). That run has **no job that checks the panic strategy**: no `-C panic=abort` build, no release-artefact panic test, no symbol check. So the pin, the `compile_error!` and the test below are still unimplemented and untested. What Miri ran was the existing FFI crate, which unwinds by default.

Facts from the language (not run here): since Rust 1.81 an unwinding panic that reaches an `extern "C"` function aborts the process, so with `unwind` and no `guard` the worst case equals `abort`. Stack overflow and out-of-memory (allocation failure) abort in both modes. Cargo ignores the `panic` profile key for the `test` profile, so `cargo test` always unwinds. On `wasm32-unknown-unknown` with stable Rust there is no unwinding, so a panic always aborts.

## What `guard` cannot stop

`guard` only turns an **unwinding** panic into a status. These failures end the process whatever the strategy, so the ADR does not promise containment for them. They are limited by documented limits and by code rules, not by `guard` (SEC-ADR-08, RISK-F-P2-08):

1. Out-of-memory and stack overflow.
2. **`panic_nounwind` aborts.** Since Rust 1.78 the standard library's unsafe-precondition checks (for example the `slice::from_raw_parts` length and alignment check) are enabled in debug builds, and a failure aborts through a path that no `catch_unwind` catches. A debug build of the FFI crate with a wrong `len` aborts the host even with `unwind`. This is the SEC-F-01 path, so the fix there (check the length **before** building the slice) is a requirement and not a nicety. Same for any `#[rustc_nounwind]` path. From language knowledge, not run here.
3. **A panic during unwinding**, for example a `Drop` that panics while another panic unwinds. It aborts. Core and FFI types therefore must keep `Drop` free of panics (item 6).
4. **A panic payload whose destructor panics.** A payload from `panic!` is a `String` or `&str` and is safe. A payload from `std::panic::panic_any` with a custom type is not. Not reachable today. Item 7 covers it.
5. **Unwinding out of `extern "C"`** aborts (Rust 1.81 and later). `extern "C-unwind"` is never used (item 8).
6. **The default panic hook writes to the host's stderr.** The text includes the source path and the panic message. Panic messages in this library must not contain caller data (threat T-27), and index values from a slice panic are caller-influenced. The library installs no hook of its own, because that is hostile to a host. SDK authors are told that stderr output can appear.

## Decision (proposed)

Use `panic = "unwind"` for `mkpdf-ffi` and the library artefacts, pinned and enforced:

1. **Pin our own artefacts.** Set `panic = "unwind"` explicitly in `[profile.release]` and `[profile.dev]` in the workspace `Cargo.toml`, with a comment that points to this ADR. This reaches only builds where this workspace is the root. A downstream crate that depends on `mkpdf-ffi` uses its own profile (Cargo behaviour from knowledge, not run here), so item 1 is not the main control.
2. **The main control: a positive `compile_error!`.** Add `#[cfg(not(panic = "unwind"))] compile_error!("mkpdf-ffi requires panic=unwind (ADR-0011)");` to `mkpdf-ffi`. A downstream `-C panic=abort` build, and any future strategy such as an `immediate-abort` mode, then fails to compile and does not silently lose the guard. There is **no `wasm32` exception**. If the FFI crate is ever built for wasm, the build fails until someone decides, which is the safe default. `mkpdf-core` is built for wasm and has no guard, and it is not affected.
3. **Checks and test.**
   - A CI check that fails when the `panic` setting in the manifest changes (secondary to item 2).
   - A CI job that builds `mkpdf-ffi` with `-C panic=abort` and **expects the build to fail** (proves item 2).
   - A panic test that runs against the **release `cdylib`** and drives a panic through a real exported function, from a separate program (a C caller or a Rust binary that loads the library). `cargo test` cannot do this because it always unwinds. The C caller test already requested in SEC-F-15 and FFI-CR-10 can be that program.
   - The test hook is gated by a custom `--cfg mkpdf_test_hooks`, set through `RUSTFLAGS` only in the test job and declared in `[lints.rust] unexpected_cfgs` so the lint stays clean. It is **not** a Cargo feature, because features are additive and any dependent or `--all-features` build can switch one on (SEC-ADR-07, RISK-F-P2-09). The hook is a non-exported function called from an export that exists only under the cfg.
   - A release-pipeline check that the shipped `cdylib` exports exactly the symbols in the header (for example with `nm`), so that no test symbol can ship.
4. **A caught panic leaves the handle poisoned.** With ADR-0013 section 4, the document mutex is held inside `guard`. A panic poisons it and every later call except `*_free` returns `MKPDF_ERR_INTERNAL` (SEC-F-03, option b). `Mutex::clear_poison` is forbidden. Until ADR-0013 is implemented, the weaker interim rule "validate first, mutate last" applies: every mutation today is a single assignment or a `push` after the limit check, which holds by inspection and is enforced by nothing.
5. **Sequencing (proposal for the Orchestrator).** No new mutating FFI export merges before ADR-0013 is implemented, because the interim rule is not enforced and a new mutating export would reopen threat T-20 (SEC-ADR-09). D-08 already puts FFI-003 before FFI-001. This extends it to every mutating function.
6. **Panic-free `Drop`.** Types that the FFI crate drops during unwinding (`MutexGuard`, `Arc`, and `Document` and everything the writer adds to it) must not panic in `drop`. Today `Document` holds a `String` and a `Vec` of plain values. That is a property to keep, not one to rely on. I propose to the owner a matching line in `rules/rust-engineering.md`. I have not edited it.
7. **Scope of `guard`.** All of the following run inside one `catch_unwind` closure, or inside a second one with a static fallback: the `Arc` clone and the lock guard of ADR-0013 (created and dropped inside), the work after a failure (`failure.message()` and `set_last_error`), and the drop of the panic payload. For the payload, either `std::mem::forget` it (a few bytes leaked per caught panic) or drop it inside its own `catch_unwind`. This moves code that today runs outside `catch_unwind` (`guard` drops the payload and calls `set_last_error` after it, FFI-CR-06). It is a change for rust-core with FFI-003b.
8. **No `extern "C-unwind"`**, anywhere. A CI grep or a clippy rule keeps it true. Devops writes it.
9. **Documentation for SDK authors:** `MKPDF_ERR_INTERNAL` means a library bug, the handle that raised it should be freed and not reused, and the process continues. Out-of-memory, stack overflow and the failures in "What `guard` cannot stop" still end the host process. The default panic hook may print to stderr.
10. `abort` is not supported for the shipped libraries. A host that wants fail-fast behaviour can terminate on `MKPDF_ERR_INTERNAL` itself.

## Alternatives

### A. `panic = "abort"` for the cdylib and staticlib

- For: smaller binaries and slightly faster code (no unwind tables, more inlining). No `UnwindSafe` question and no half-mutated state after a panic. Fail-fast on a bug.
- Against: every panic is a host crash. The host is an in-process Node, Python, .NET, Java or Go application, often a server, so one bad document kills every other request (denial of service). The `guard` and `catch_unwind` code, the `panic_is_contained` test and threat T-01's mitigation would be dead or removed, and `MKPDF_ERR_INTERNAL` would never be returned. A crash inside a foreign runtime gives a poor diagnostic.
- Also: it does not remove the need for no-panic discipline. It only changes the consequence.
- Rejected, because the failure mode is worse for the stated hosts, and the size gain is not shown to matter.

### B. `unwind`, but `guard` on some functions only

Rejected. Partial coverage cannot be reviewed, and `rules/rust-engineering.md` item 5 asks for all of them.

### C. `abort` in the core and `unwind` in the FFI crate

Not possible per crate. The strategy is chosen by the final artefact's profile, not by individual crates in a workspace.

### D. Leave it undecided and rely on the default

Rejected. This is the current state. It is not tested and not enforced, and a downstream rebuild can break it without notice (SEC-F-14).

### E. Pin only through the manifest, with no `compile_error!`

Rejected. A profile in this workspace does not reach a downstream build, so the pin would give no protection where the risk is. The `compile_error!` does.

## Consequences

- Panic containment (threat T-01) **will be** a tested property of the shipped library once items 2 and 3 exist and their tests pass. Today it is tested for `guard` only (`panic_is_contained`), and CI run 45103722429 has no check of the panic strategy. T-01 stays at its current status until then.
- The FFI crate gains a `compile_error!`, a CI job that expects a failed build, a release-artefact panic test and a symbol check. These are tasks for rust-core and devops, none of them done.
- Binaries are somewhat larger, by an amount to be measured on the real writer before v0.1.0.
- Code in the FFI crate must be written so that a caught panic cannot leave shared state broken (item 4), and so that nothing that runs outside `guard` can panic (items 6 and 7).
- Callbacks into foreign code (a streaming sink) would add foreign unwinding into Rust frames, which `unwind` does not make safe by itself. ADR-0012 does not use callbacks for that reason. A later callback ADR must address it.
- On `wasm32` the FFI crate does not build under this ADR (no exception, item 2). The core builds for wasm and has no guard, and a panic there aborts the module. WASM bindings must say so.
- Needs a Security Engineer review of this amended text before acceptance.

## Still open (NOT DECIDED)

- Acceptance of this ADR. The owner decides.
- Size and run-time measurements on a realistic build.
- Whether the owner approves the two rules-file lines proposed above (panic-free `Drop`, and no `C-unwind`).

## Not examined

- The facts from the first draft were not re-run by the Security Engineer. Only the positive `cfg` form was run again for this amendment (see Context).
- The `panic_nounwind` behaviour, the abort on a panic during unwinding, the effect of profiles on a downstream build and the `panic_any` payload behaviour are from knowledge of Rust and were not run here.
- Linux and Windows behaviour of the `compile_error!` and of the release-artefact test is not examined.
