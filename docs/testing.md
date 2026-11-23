# Testing strategy (QA-001)

Rules: `rules/testing.md`. This document gives the plan.

| Level | Tool | Scope | State |
|---|---|---|---|
| Unit | `cargo test` | Core validation and model (4 tests) | Running locally |
| FFI | `cargo test -p mkpdf-ffi` | Null, UTF-8, invalid values, panic containment, lifecycle (6 tests) | Running locally |
| Memory | Miri (nightly) on `mkpdf-ffi` | Undefined behaviour in unsafe code | CI job defined, never run |
| Integration | `tests/` directories | Public API end to end | None yet |
| Conformance | qpdf, poppler, later veraPDF | Generated PDFs, as black boxes | Needs PDF output, and qpdf is not installed here |
| Fuzz | cargo-fuzz | Serialiser inputs, later the parser | Not started (FUZZ-001) |
| Regression | Named tests per bug | | None needed yet |
| Determinism | Byte equality of two runs | Writer | Not started |
| Cross-platform | CI matrix | Linux, macOS, Windows | CI defined, never run. Only macOS (arm64) has been tested, and only by hand |
| Language binding | One test suite per SDK | Create a multi-page document through the SDK | Not started (v0.2.0) |

## Exit criteria for v0.1.0

- All gates in `AGENTS.md` pass in CI on three operating systems.
- A multi-page PDF from the Rust API and from the C ABI passes `qpdf --check` and opens in poppler without warnings, with tool versions recorded.
- Miri passes on `mkpdf-ffi`.
- A fuzz target for the serialiser ran for its time budget without a crash.
- No claim of standards conformance beyond the validators that ran.
