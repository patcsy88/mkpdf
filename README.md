# mkpdf

An open-source PDF generation and processing engine written in Rust, with a stable C ABI so that applications in other languages can use the same core.

**Status: foundation milestone. mkpdf cannot produce a PDF file yet.** What exists is the workspace, the document model (pages and a title), a C ABI skeleton, rules and documentation. PDF serialisation is the first task of v0.1.0 (see `docs/roadmap.md`).

## Goals

- Rust core, safe by default. `unsafe` is confined to the FFI crate.
- One engine for every language. SDKs for Node.js, Python, .NET, C++ and Java consume the C ABI and do not reimplement PDF logic.
- Offline, with no external service and no commercial licence.
- Apache-2.0.

No claim of PDF, PDF/A or PDF/UA conformance is made. Such a claim will name the validator that supports it.

## Layout

| Path | Purpose |
|---|---|
| `crates/mkpdf-core` | Document model and engine |
| `crates/mkpdf-ffi` | C ABI |
| `crates/mkpdf-cli` | Command line (`mkpdf --version` only) |
| `rules/` | Engineering rules |
| `docs/` | Architecture, ADRs, API specs, threat model, roadmap |

## Build

```
cargo build --workspace
cargo test --workspace
```

## Documentation

- `docs/architecture.md`
- `docs/roadmap.md`
- `docs/requirements.md` (decisions and open questions)
- `docs/api/rust-api.md`, `docs/api/c-abi.md`
- `docs/security/threat-model.md`
- `CONTRIBUTING.md`, `SECURITY.md`, `AGENTS.md`

## Licence

Apache-2.0. See `LICENSE` and `NOTICE`.
