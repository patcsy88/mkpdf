# Threat model (SEC-001)

Status: draft, not yet reviewed (GATE-1). Scope: the foundation milestone plus the planned v0.1.0 writer. Parsing, encryption and signatures are listed so the design leaves room for them, and need their own update before they are built.

## Assets

- Integrity of the host process (no memory corruption, no crash) when mkpdf is loaded in-process by a Node, Python, .NET, Java or C++ application.
- Availability of the host (bounded CPU and memory).
- Confidentiality of document content passed through the library.
- Correctness of generated PDFs.
- Supply-chain integrity of the source, dependencies and release artefacts.

## Trust boundaries

| ID | Boundary | Untrusted side |
|---|---|---|
| TB-1 | C ABI | Every pointer, length, string and call order from the caller |
| TB-2 | Rust API | Values passed by Rust callers (validated, but memory-safe by construction) |
| TB-3 | Input files (v0.4.0) | PDFs, fonts and images from outside |
| TB-4 | Build and release pipeline | Dependencies, CI actions, prebuilt SDK binaries |
| TB-5 | Server interfaces (later) | Network clients |

## Threats and mitigations

| ID | Threat | Boundary | Mitigation | Status |
|---|---|---|---|---|
| T-01 | Panic unwinds across the FFI into a foreign runtime (undefined behaviour) | TB-1 | `catch_unwind` in `guard`; `free` also guarded; test `panic_is_contained`. Add `panic = "abort"` review (see note) | Implemented, tested |
| T-02 | Null or dangling pointer from caller | TB-1 | Null checks on every pointer argument and tests. Dangling pointers cannot be detected, and the contract says so | Partly mitigated |
| T-03 | Double free or use after free of a handle | TB-1 | Contract in `docs/api/c-abi.md`; `free(NULL)` is a no-op. No runtime detection yet. Option for GATE-1: generation-checked handles | Open (accepted risk pending decision) |
| T-04 | Resource exhaustion through huge page counts or sizes | TB-1, TB-2 | `MAX_PAGES`, `MAX_PAGE_DIMENSION_PT`, validated at insertion. More limits for content, images and fonts when they exist | Partly mitigated |
| T-05 | Invalid UTF-8 in strings | TB-1 | Validated with `from_utf8`; returns `MKPDF_ERR_INVALID_UTF8` | Implemented, tested |
| T-06 | Non-finite or hostile numbers (NaN, infinity) in geometry | TB-1, TB-2 | Rejected in `Page::new` | Implemented, tested |
| T-07 | Data race by sharing a handle across threads | TB-1 | Contract: one thread at a time per handle. No internal locking. Documented | Documented |
| T-08 | Stale error pointer read after the next call | TB-1 | Documented lifetime. Error text stored per thread | Documented |
| T-09 | Integer overflow in sizes and offsets (writer, parser) | TB-2, TB-3 | Overflow checks on in release, checked arithmetic rule, tests at boundaries | Rule set, not yet testable |
| T-10 | Decompression bombs and deep nesting | TB-3 | Output-size and depth limits, fuzzing, before parsing ships | Planned (v0.4.0) |
| T-11 | Malicious font or image triggers a parser flaw | TB-3 | Safe Rust, limits, fuzz targets, choice of crates reviewed | Planned |
| T-12 | Vulnerable or malicious dependency | TB-4 | Allowlist (ADR-0005), `cargo deny`, `cargo audit`, lockfile committed for binaries, review | Policy set. CI unrun |
| T-13 | Compromised CI action or release artefact | TB-4 | Pin actions to commit SHAs before release, least-privilege tokens, reproducible builds, signed releases | Planned |
| T-14 | PDF output that triggers a vulnerability in a viewer (for example active content) | output | Emit no JavaScript, launch actions or external references unless a feature asks for it. Document that | Rule set |
| T-15 | Information leak through metadata (usernames, paths, timestamps) | output | The core adds none by default (ADR-0007) | Design |
| T-16 | Weak cryptography or wrong signatures | later | Reviewed crates only, ADR and security review per feature | Planned (v0.5.0) |
| T-17 | Server interface abuse (SSRF, request size) | TB-5 | Out of scope until those interfaces are designed | Deferred |

## Notes and open items

1. **Panic strategy.** `catch_unwind` only works with `panic = "unwind"`. If the project later sets `panic = "abort"`, a panic aborts the host process, which still does not cross the boundary but is a denial of service. The choice needs an ADR before v0.1.0.
2. **Handle validation (T-03).** Decide at GATE-1 whether to accept the C-style contract or add checked handles.
3. **Supply chain.** `cargo-audit` is not installed locally, and no CI run exists yet. T-12 and T-13 have not been tested.
4. **No fuzzing yet.** There is no parser or serialiser to fuzz at this point.
5. This model has had no independent review.
