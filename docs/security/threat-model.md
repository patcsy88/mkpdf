# Threat model (SEC-001)

Status: draft. First security review done on 2026-11-23 (`docs/reviews/gate-1-security.md`, GATE-1). The review is by the Security Engineer role only, with no independent second reviewer. Scope: the foundation milestone plus the planned v0.1.0 writer. Parsing, encryption and signatures are listed so the design leaves room for them, and need their own update before they are built.

## Assets

- Integrity of the host process (no memory corruption, no crash) when mkpdf is loaded in-process by a Node, Python, .NET, Java or C++ application.
- Availability of the host (bounded CPU and memory).
- Confidentiality of document content passed through the library.
- Correctness of generated PDFs.
- Supply-chain integrity of the source, dependencies and release artefacts.

## Trust boundaries

| ID | Boundary | Untrusted side |
|---|---|---|
| TB-1 | C ABI | Every pointer, length, string and call order from the caller. Includes bugs in our own SDK wrappers (for example a signed-to-unsigned length conversion) |
| TB-2 | Rust API | Values passed by Rust callers (validated, but memory-safe by construction) |
| TB-3 | Input files (v0.4.0) | PDFs, fonts and images from outside |
| TB-4 | Build and release pipeline | Dependencies, CI actions, prebuilt SDK binaries, package names on public registries |
| TB-5 | Server interfaces (later) | Network clients |
| TB-6 | CLI (from v0.1.0) | Command-line arguments, output paths, symlinks, environment. Added by the GATE-1 review |

## Threats and mitigations

Status values: Implemented (code exists), Tested (a test or run shows it works), Documented, Planned, Open. A row is not "Tested" unless the evidence is named.

| ID | Threat | Boundary | Mitigation | Status |
|---|---|---|---|---|
| T-01 | Panic unwinds across the FFI into a foreign runtime. Since Rust 1.81 an unwinding `extern "C"` function aborts the process, so the effect is a host crash, not silent UB. `guard` turns it into `MKPDF_ERR_INTERNAL` | TB-1 | `catch_unwind` in `guard`; `free` also guarded; test `panic_is_contained` covers `guard` only. Depends on `panic = "unwind"` (OQ-12, ADR-0011 pending). Three exported functions bypass `guard` on purpose (see SEC-F-13) | Implemented. Tested for `guard` only. Not tested per exported function. Miri unrun (SEC-F-05) |
| T-02 | Null or dangling pointer from caller | TB-1 | Null checks on every pointer argument and tests. Dangling pointers cannot be detected, and the contract says so | Partly mitigated |
| T-03 | Double free or use after free of a handle | TB-1 | Contract in `docs/api/c-abi.md`; `free(NULL)` is a no-op. No runtime detection. A magic tag does not help against use after free, because reading freed memory is itself UB. Only a handle table with generation counters detects it | Open. Needs an owner decision (SEC-F-04) |
| T-04 | Resource exhaustion through huge page counts, sizes or strings | TB-1, TB-2 | `MAX_PAGES`, `MAX_PAGE_DIMENSION_PT`, validated at insertion. **No limit on title length** (SEC-F-02). Allocation failure aborts the process and cannot be caught. More limits for content, images and fonts when they exist | Partly mitigated |
| T-05 | Invalid UTF-8 in strings | TB-1 | Validated with `from_utf8`; returns `MKPDF_ERR_INVALID_UTF8`. A valid string may contain U+0000, so the serialiser must escape it | Implemented, tested (`invalid_inputs_set_error_and_success_clears_it`) |
| T-06 | Non-finite or hostile numbers (NaN, infinity) in geometry | TB-1, TB-2 | Rejected in `Page::new` | Implemented, tested |
| T-07 | Data race by sharing a handle across threads. `&` and `&mut` are formed from the raw pointer, so a race is UB | TB-1 | Contract: one thread at a time per handle. No locking and no detection | Documented, not enforced (SEC-F-11) |
| T-08 | Stale error pointer read after the next call | TB-1 | Documented lifetime. Error text stored per thread | Documented |
| T-09 | Integer overflow in sizes and offsets (writer, parser) | TB-2, TB-3 | `overflow-checks = true` in the workspace release profile, checked arithmetic rule, tests at boundaries. A downstream build that ignores our profile loses the checks | Rule set, not yet testable |
| T-10 | Decompression bombs, deep nesting, stack overflow in recursion or `Drop` of nested structures (a stack overflow aborts and cannot be caught) | TB-3 | Output-size and depth limits, fuzzing, before parsing ships | Planned (v0.4.0) |
| T-11 | Malicious font or image triggers a parser flaw | TB-3 | Safe Rust, limits, fuzz targets, choice of crates reviewed | Planned |
| T-12 | Vulnerable or malicious dependency | TB-4 | Allowlist (ADR-0005), `cargo deny`, `cargo audit`, lockfile committed for binaries, review | `cargo deny check` ran locally on 2026-11-23 and passed, but the workspace has no third-party dependency, so it checked nothing yet. `cargo audit` not installed, CI never run |
| T-13 | Compromised CI action or release artefact | TB-4 | Pin actions to commit SHAs, least-privilege tokens, reproducible builds, signed releases. Current CI uses mutable tags and branches and an unpinned `cargo install cargo-audit` (SEC-F-07) | Open. Permissions are already `contents: read` |
| T-14 | PDF output that triggers a vulnerability in a viewer (for example active content) | output | Emit no JavaScript, launch actions or external references unless a feature asks for it. Document that | Rule set |
| T-15 | Information leak through metadata (usernames, paths, timestamps) in the PDF, or build paths in release binaries | output, TB-4 | The core adds no metadata by default (ADR-0007). Build-path leakage in Linux release artefacts is not examined (SEC-F-18) | Design. Partly examined |
| T-16 | Weak cryptography or wrong signatures | later | Reviewed crates only, ADR and security review per feature | Planned (v0.5.0) |
| T-17 | Server interface abuse (SSRF, request size) | TB-5 | Out of scope until those interfaces are designed | Deferred |
| T-18 | Header and implementation drift. `include/mkpdf.h` is hand-written, so a wrong signature gives UB in every caller | TB-1 | Today the 8 exported symbols match the header (checked by `nm` and by reading). Generate with cbindgen and diff in CI (interoperability rule 8) | Open (SEC-F-15) |
| T-19 | Length or size from a managed runtime is wrong (negative `int` converted to `size_t`, so a huge `len`). `slice::from_raw_parts` with `len > isize::MAX` is UB | TB-1 | Reject `len > isize::MAX` before building the slice, and add a string size limit | Open (SEC-F-01) |
| T-20 | Handle left inconsistent after a caught panic, then reused | TB-1 | Decide in ADR-0011: mark the handle poisoned, or state that it may only be freed after `MKPDF_ERR_INTERNAL`. Core operations are atomic with respect to panics today | Open (SEC-F-03) |
| T-21 | Wrong or lost error text when a runtime moves a call between OS threads (Go goroutines, async runtimes), or when the library is unloaded while thread-local destructors are registered | TB-1 | Document the pinning requirement. Consider an API that copies the message into a caller buffer. Test unload on each platform | Open (SEC-F-12) |
| T-22 | Out parameters left uninitialised on failure, so a caller that ignores the status uses garbage as a handle | TB-1 | Write null to `*out` on every path | Open (SEC-F-10) |
| T-23 | Output buffer allocator mismatch or double free (FFI-001): a buffer allocated by Rust and freed by the caller's `free` | TB-1 | Decided by ADR-0012 (pending). Needs review before FFI-001 merges | Planned |
| T-24 | Reentrancy through callbacks or streaming writers (FFI-001): a callback calls back into the same handle while a `&mut` is live | TB-1 | Forbid reentry on a handle, or detect it, in the ADR for streaming output | Planned |
| T-25 | Disclosure failure: no private channel for reports (OQ-01), so a reporter goes public | process | Owner enables GitHub private reporting and sets the alias before the first public release | Open (SEC-F-09) |
| T-26 | Name squatting or dependency confusion for the package names on crates.io, npm, PyPI, NuGet and Maven (OQ-02 not done) | TB-4 | Reserve the names, use scoped names, verify publisher accounts with 2FA | Open (owner action) |
| T-27 | Document content kept in freed memory, or copied into error messages, logs or panic output | TB-1 | Error messages must not echo caller strings. Core messages today contain only static text and constants. Consider zeroising buffers for sensitive modes later | Documented. Low risk today |
| T-28 | CLI misuse (v0.1.0): output path overwrite, symlink following, reading from untrusted paths | TB-6 | The CLI is the only crate allowed file I/O (rules/security.md 5). Define behaviour (no follow, explicit overwrite flag) before the first file-writing command | Planned |

## Notes and open items

1. **Panic strategy.** `catch_unwind` only works with `panic = "unwind"`. If the project later sets `panic = "abort"`, a panic aborts the host process, which still does not cross the boundary but is a denial of service. The choice needs an ADR before v0.1.0 (ADR-0011 is being drafted by another agent and was not reviewed here).
2. **Handle validation (T-03).** Decision for the project owner at GATE-1: accept the C-style contract, or add checked handles. See SEC-F-04.
3. **Supply chain.** `cargo-audit` is not installed locally, and no CI run exists yet. T-12 and T-13 have not been tested in CI.
4. **No fuzzing yet.** There is no parser or serialiser to fuzz at this point.
5. **Miri not run.** Not available locally (no rustup, no nightly). The CI job exists but may not run on nightly (SEC-F-06).
6. This model has had one review, by the Security Engineer role. It has not had an independent review by a second person. Only the project owner can accept a risk, in writing in the repository. No risk acceptance is recorded yet.
