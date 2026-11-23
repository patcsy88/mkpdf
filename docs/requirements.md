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
| D-08 | Handle safety (SEC-F-04) | Build generation-checked handles, so a double free or use after free is detected and returns an error and not undefined behaviour. This amends ADR-0003 and changes the ABI. It needs a new ADR, and the work comes before FFI-001 (backlog item FFI-003). The ABI is 0.x, so the change is allowed now |
| D-09 | Risk Reviewer agent | Approved on 2026-11-23 as a tenth agent (OQ-14). It also reviews GATE-1 as the independent second reviewer. It never accepts a risk |
| D-10 | Push timing | Push the CI toolchain fix and the SEC-F-01/02 fixes after the Security Engineer's re-review is in. Commit in logical groups with `-s`. Push only when the owner says "push" for each push, unless the owner states otherwise (2026-11-23: "push after the re-review") |
| D-11 | Handle table design (FFI-003) | Table indices with a generation counter, so a forged or stale handle is never dereferenced. The architect writes the ADR before FFI-001. The ADR stays Proposed until the owner accepts it |
| D-13 | Handle token type (ADR-0013 point 1) | `uint64_t` token, not a fake pointer. Token 0 is never valid |
| D-14 | Out parameters on failure (ADR-0013 point 2) | A neutral value is written first: `usize::MAX` for a page index, 0 for tokens and counts |
| D-15 | Concurrent use of one handle (ADR-0013 point 3) | Return `BUSY`, do not block |
| D-16 | ABI 0.x versioning (ADR-0013 point 4, ARCH-F-03) | Exact major and minor match while 0.x, and the ABI moves to 0.2 with the handle change. ADR-0004 and `rules/interoperability.md` item 7 need amending (not done yet) |
| D-17 | Status codes (ADR-0013 point 5) | 5 = `INVALID_HANDLE`, 6 = `BUSY`. `free(0)` is a no-op returning OK |
| D-18 | `free` on a handle in use (ADR-0013 point 7) | `free` succeeds and the running call finishes |
| D-19 | `MAX_LIVE_HANDLES` and ADR-0013 acceptance (points 6 and 8) | The Security Engineer proposes the value and the owner accepts it. ADR-0013 stays Proposed until the Security Engineer has reviewed it. Then the owner accepts it. FFI-003 starts after that |
| D-12 | Output determinism (OQ-10) | Deterministic by default: no timestamp and no random document ID. Timestamps and IDs are opt-in. ADR-0007 must be updated to say so, and it stays Proposed until accepted |

## Resolved questions (D-07)

| ID | Decision | State |
|---|---|---|
| OQ-01 | Security reports through GitHub private vulnerability reporting. Conduct reports through a role alias on a project-owned domain, not a personal address | **Security side done:** GitHub private vulnerability reporting enabled on 2026-11-23 and `SECURITY.md` updated. **Deferred by owner (2026-11-23):** the conduct alias is to be created later, and the placeholder in `CODE_OF_CONDUCT.md` stays until then. It must be set before any release |
| OQ-02 | Choose the final organisation and repository name before publishing. Check that `mkpdf` is free on crates.io, npm, PyPI, NuGet and Maven | **Decided (2026-11-23): move to a GitHub organisation.** **Pending (owner):** choose the organisation name, create it and transfer the repository. `Cargo.toml`, `SECURITY.md` and the README links then need updating. Until then `github.com/patcsy88/mkpdf` is a temporary home. The registry availability check has not been run |
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
