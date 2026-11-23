# Requirements analysis (REQ-001)

Source: the project brief. This document records the decisions taken and the open questions.

## Decisions taken with the project owner

| ID | Decision | Choice |
|---|---|---|
| D-01 | Backlog | Adopt the revised backlog with missing items, verifiable criteria and fixed ordering (docs/roadmap.md) |
| D-02 | C ABI handle model | Opaque handles. The library owns every object, with explicit create and free functions. Status codes plus a thread-local last-error message |
| D-03 | Third-party crates in the core | A small allowlist of permissively licensed crates. Each crate needs an ADR and a licence and security review |
| D-04 | Contribution sign-off | DCO (`Signed-off-by`) |
| D-05 | Release progression | The owner's six releases, kept as proposed (docs/roadmap.md) |
| D-06 | Go SDK | Added to v0.2.0 through CGo over the C ABI. gRPC is not used for Go |
| D-07 | Resolution of OQ-01 to OQ-09 | The recommendations were accepted. See the table below. Items marked "action pending" are not finished |

## Resolved questions (D-07)

| ID | Decision | State |
|---|---|---|
| OQ-01 | Security reports through GitHub private vulnerability reporting. Conduct reports through a role alias on a project-owned domain, not a personal address | **Security side done:** GitHub private vulnerability reporting enabled on 2026-11-23 and `SECURITY.md` updated. **Action pending (owner):** create the conduct alias and replace the placeholder in `CODE_OF_CONDUCT.md` |
| OQ-02 | Choose the final organisation and repository name before publishing. Check that `mkpdf` is free on crates.io, npm, PyPI, NuGet and Maven | **Partly done:** repository is `github.com/patcsy88/mkpdf` and `Cargo.toml` now points to it. **Pending:** whether to move to an organisation, and the registry availability check, which has not been run |
| OQ-03 | Tier 1 (CI-tested, release-blocking): Linux x86_64 and aarch64, macOS arm64, Windows x86_64. Best effort: macOS x86_64, musl, `wasm32` | Decided. Not yet true: only macOS arm64 has been tested, by hand. The CI matrix covers three runners and has never run. It needs a Linux aarch64 job |
| OQ-04 | Minimum Rust version stays 1.85, the floor for edition 2024. It is raised only through a recorded decision | Decided. The CI MSRV job has not run, so it is unverified |
| OQ-05 | Candidates to evaluate: `miniz_oxide` (via `flate2`) for Flate, `ttf-parser` or `fontations` for fonts, `subsetter`, `rustybuzz` | **No crate is approved.** The names are from memory. Each needs the review in `rules/licensing.md`. Flate is needed first, fonts and shaping later |
| OQ-06 | v0.1.0 includes vector graphics, images and text in the 14 standard fonts with a limited character set. Embedded fonts and full Unicode stay in v0.3.0 | Decided. Standard-font text is not self-contained, so no PDF/A claim is possible for it. JPEG passthrough needs no decoder |
| OQ-07 | "Templates" means a declarative document description (JSON) with placeholder substitution. No scripting, expressions, network or file access. Deferred if it cannot be defined | Decided in outline. A definition is needed before v0.3.0 planning |
| OQ-08 | Watermarks in v0.4.0. PDF/A in v0.5.0 or just before v1.0.0, only if veraPDF runs in CI. Markdown conversion in v0.3.0 if layout is ready. PDF/UA and HTML conversion after v1.0.0 | Decided. Placement is conditional as stated |
| OQ-09 | Basic CLI and a `wasm32` build check in CI from v0.1.0. WASM bindings with v0.2.0. REST and gRPC as separate crates after v0.4.0, outside v1.0 unless the owner says otherwise | Decided. The wasm32 target is not installed here, so the check is unrun |

## Open questions

| ID | Question | Why it matters | Needed by |
|---|---|---|---|
| OQ-10 | Is the PDF output deterministic by default (no timestamp, no random document ID)? | The brief wants determinism "where supported" | ADR-0007 |
| OQ-11 | How will the validators that are not installed (qpdf, veraPDF, cbindgen, cargo-audit) be provided? | Without them conformance cannot be claimed. Installing them locally needs the owner's approval | Before PDF-002 |
| OQ-12 | What is the panic strategy (`unwind` or `abort`)? | The panic guard depends on `unwind` (threat T-01) | Before v0.1.0 |
| OQ-13 | How is the output buffer owned across the C ABI? | Needed before FFI-001 | Before FFI-001 |

## Architectural decisions identified

Each one gets an ADR (docs/adr/): workspace layout, safe-Rust policy and the single `unsafe` crate, C ABI handle model, error model, PDF version strategy, determinism, resource limits, dependency policy, binding strategy (every SDK consumes only the C ABI), WASM constraints.

## Constraints from the brief that are enforced mechanically

- `unsafe_code = "deny"` in the workspace, `forbid` in `mkpdf-core` and `mkpdf-cli`. Only `mkpdf-ffi` allows it.
- No `unwrap`, `expect`, `panic`, `todo` or `unimplemented` in library code (clippy).
- Every FFI function runs inside a panic guard.
- `cargo-deny` enforces the licence allowlist, banned wildcards and the registry source.
- No claim of standards compliance without a named validator run.
