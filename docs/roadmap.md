# Roadmap and prioritised backlog

Status: draft, pending the architecture and security review gate (GATE-1).

## Release progression (approved by project owner)

| Release | Name | Scope |
|---|---|---|
| v0.1.0 | Foundation | Rust core, PDF writer, document model and C ABI |
| v0.2.0 | Language integration | Node.js, Python, .NET, C++, Java and Go (CGo) SDKs, WASM bindings |
| v0.3.0 | Document layout | Tables, pagination, Unicode and templates. Markdown conversion if layout is ready |
| v0.4.0 | PDF processing | Merge, split, rotate, rendering, extraction and watermarks |
| v0.5.0 | Advanced capabilities | Forms, encryption and digital signatures |
| v1.0.0 | Production release | Stable API, supported platforms, documented limitations, security review and release conformance |

Before v0.1.0 there are pre-releases (v0.0.x) covering the foundation milestone in the next section.

## Scope decisions

Resolved on 2026-11-23 (D-06, D-07 in `docs/requirements.md`). Items still conditional or unfinished are marked.

| ID | Capability | Decision |
|---|---|---|
| SCOPE-01 | Text, fonts, images, vector graphics | v0.1.0 includes vector graphics, images (JPEG passthrough) and text in the 14 standard fonts with a limited character set. Embedded fonts and full Unicode stay in v0.3.0. Standard-font text is not self-contained, so no PDF/A claim is possible for it |
| SCOPE-02 | Page configuration, metadata, headers and footers | Page size and metadata in v0.1.0. Headers and footers in v0.3.0 |
| SCOPE-03 | Watermarks | v0.4.0, since they need a parser |
| SCOPE-04 | PDF/A | v0.5.0 or just before v1.0.0, **only if veraPDF runs in CI**. PDF/UA is after v1.0.0 |
| SCOPE-05 | HTML and Markdown conversion | Markdown in v0.3.0 if layout is ready. HTML after v1.0.0 |
| SCOPE-06 | CLI, REST, gRPC, WASM | Basic CLI and a `wasm32` build check in CI from v0.1.0. WASM bindings in v0.2.0. REST and gRPC as separate crates after v0.4.0, outside v1.0 unless the owner says otherwise |
| SCOPE-07 | Templates | A declarative JSON document description with placeholder substitution. No scripting, expressions, network or file access. **Needs a written definition before v0.3.0 planning. Deferred if it cannot be defined** |
| SCOPE-08 | v0.2.0 breadth | Six SDKs (with Go) and WASM bindings in one release. If it slips, ship them one at a time as v0.2.x without changing the order of the other releases. |
| SCOPE-09 | v0.4.0 breadth | Parsing, rendering and extraction in one release. The parser is the main attack surface and needs fuzzing before release. |

## Backlog

Priority: P0 blocks everything after it, P1 is required for the release, P2 is desirable.

### Phase 0: foundation (v0.0.x)

| ID | Deliverable | Acceptance criteria | Pri | Depends on | Status |
|---|---|---|---|---|---|
| REQ-001 | Requirements analysis and open-questions log | `docs/requirements.md` lists decisions and open questions, with the owner's answers recorded | P0 | none | Done |
| ARCH-001 | High-level architecture | `docs/architecture.md` defines components and boundaries and passes GATE-1 with the review recorded in the repo | P0 | REQ-001 | Draft |
| ARCH-002 | Architecture decision records | One ADR per significant decision, indexed in `docs/adr/README.md` | P0 | ARCH-001 | Draft |
| ARCH-003 | Cargo workspace | `cargo fmt --check`, `cargo clippy -D warnings` and `cargo test` pass on the workspace | P0 | ARCH-001 | Done (macOS only) |
| ARCH-004 | PDF 1.7 and 2.0 strategy | An ADR covers the version target, the header and catalog version handling, and the 2.0 feature gating | P0 | ARCH-001 | Draft |
| SEC-001 | Threat model | `docs/security/threat-model.md` lists assets, trust boundaries, threats and mitigations | P0 | ARCH-001 | Draft |
| GOV-001 | Project rules | All seven rule files exist in `rules/` and are referenced by AGENTS.md | P0 | none | Draft |
| GOV-002 | Governance files | AGENTS.md, LICENSE, NOTICE, CONTRIBUTING.md, SECURITY.md, CODE_OF_CONDUCT.md, README.md, CHANGELOG.md exist | P0 | none | Draft |
| LIC-001 | Licence and dependency policy | Apache-2.0, `deny.toml`, `docs/dependencies.md` allowlist and provenance record | P0 | none | Draft |
| AGT-001 | Agent configuration | Ten agents (the nine required plus the Risk Reviewer approved under OQ-14), each with responsibilities, scope, permissions, deliverables, quality requirements and escalation rules | P0 | GOV-001 | Draft |
| QA-001 | Testing strategy | Unit, integration, fuzz, conformance and regression plans in `rules/testing.md` and `docs/testing.md` | P0 | ARCH-001 | Draft |
| CICD-001 | CI/CD | Workflows for fmt, clippy, test, deny, audit and the Tier 1 matrix (Linux x86_64 and aarch64, macOS arm64, Windows x86_64). Each job is run at least once. The Linux aarch64 runner (`ubuntu-24.04-arm`) is in the workflow matrix but unrun, and its availability for this repository is unconfirmed | P0 | ARCH-003 | Draft (unrun) |
| OWN-001 | Owner actions | Enable GitHub private vulnerability reporting, create the conduct alias, choose the organisation and repository name, check `mkpdf` name availability on the five registries (OQ-01, OQ-02) | P0 | none | Pending (owner) |
| FFI-003 | Generation-checked handles (D-08, SEC-F-04) | An ADR amends ADR-0003. Handles are validated on every call. Double free, use after free and foreign pointers return an error status and never cause undefined behaviour. Tests cover each case, and Miri passes. The ABI and header change and the ABI minor version is raised | P0 | DEC-001 | Not started |
| DEC-001 | Panic strategy and output-buffer ownership | ADRs for OQ-12 and OQ-13 | P0 | ARCH-001 | Not started |
| API-001 | Document API specification | `docs/api/rust-api.md` matches the code in `mkpdf-core` | P0 | ARCH-001, SEC-001 | Draft |
| API-002 | C ABI specification | `docs/api/c-abi.md` and `include/mkpdf.h` state ownership, error, threading and versioning rules | P0 | SEC-001 | Draft |
| GATE-1 | Architecture and security review | A written review with findings and dispositions. No open critical or high finding | P0 | all above | Not started |

### Phase 1: v0.1.0 Foundation

| ID | Deliverable | Acceptance criteria | Pri | Depends on |
|---|---|---|---|---|
| PDF-001 | Object model and serialiser | Objects, indirect references, a cross-reference table and a trailer. Output deterministic for equal input | P0 | GATE-1 |
| PDF-002 | Multi-page document | N pages with a page tree. Output passes `qpdf --check` and opens in poppler without warnings | P0 | PDF-001 |
| PDF-003 | Content streams and Flate compression | Compression through an allowlisted crate (OQ-05, none approved yet). Decompression is limited in size | P1 | PDF-001 |
| PDF-004 | Vector graphics, images and standard-font text | Paths, JPEG passthrough and text in the 14 standard fonts with a limited character set (SCOPE-01). Output passes the PDF-002 validators | P1 | PDF-003 |
| CLI-001 | Basic CLI | `mkpdf` can write a generated document to a file | P1 | PDF-002 |
| WASM-001 | wasm32 build check | CI builds `mkpdf-core` for `wasm32-unknown-unknown`. The target is not installed locally | P1 | ARCH-003 |
| FFI-001 | C ABI for documents and serialisation | Functions to save to a caller buffer or a callback. Header generated and checked in CI | P0 | PDF-002, API-002 |
| FFI-002 | Memory and panic safety tests | Null, double-free-adjacent, invalid UTF-8 and panic tests. Miri run on the FFI crate | P0 | FFI-001 |
| QA-002 | Conformance harness | Generated PDFs are validated by qpdf and poppler in CI | P0 | PDF-002 |
| FUZZ-001 | Fuzz targets | A target for the serialiser input path. Runs in CI for a fixed time budget | P1 | PDF-002 |

### Phase 2 onward

Items for v0.2.0 to v1.0.0 are created when the previous release's review gate closes. The first entries are:

| Release | Items |
|---|---|
| v0.2.0 | SDK-CPP, SDK-PY, SDK-NODE, SDK-NET, SDK-JAVA, SDK-GO (CGo), WASM bindings. Each has its own integration tests and consumes only the C ABI |
| v0.3.0 | Fonts and Unicode, paragraph layout, tables, pagination, headers and footers, templates (after SCOPE-07), Markdown conversion if layout is ready |
| v0.4.0 | Parser (fuzzed first), merge, split, rotate, extraction, watermarks, renderer |
| v0.5.0 | Forms, encryption and signatures. Each needs a cryptography dependency ADR and a security review |
| v1.0.0 | ABI freeze, platform support matrix, limitations document, final security review, conformance report |

## Tooling status

| Tool | Local | Needed for |
|---|---|---|
| rustc, cargo, rustfmt, clippy | Installed | everything |
| cargo-deny | Installed | LIC-001, CICD-001 |
| pdfinfo (poppler) | Installed | QA-002 |
| cargo-audit | Missing | CICD-001 (CI installs it) |
| cbindgen | Installed (0.29.4) | FFI-001 |
| qpdf | Installed (12.4.2) | PDF-002, QA-002 |
| veraPDF | Missing | SCOPE-04 |
| Miri (nightly) | Not checked | FFI-002 |
