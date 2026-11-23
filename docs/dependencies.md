# Dependency and provenance record

Policy: `rules/licensing.md` and ADR-0005. Machine check: `deny.toml`.

## Runtime dependencies

None. `mkpdf-core`, `mkpdf-ffi` and `mkpdf-cli` depend only on the Rust standard library and on each other.

## Candidate additions (not approved)

| Need | Candidate | Status |
|---|---|---|
| Flate compression | To be selected (OQ-05) | Not reviewed |
| Font parsing and subsetting | To be selected | Not reviewed |
| Text shaping | To be selected | Not reviewed |
| Hashing and cryptography | To be selected for v0.5.0 | Not reviewed |

## Review template for a new crate

Name and version. Licence (SPDX). Maintainers and last release. Transitive crates. Advisory history (`cargo audit`). Why it is needed, and what not using it would cost. `unsafe` usage. Build scripts. Reviewer and date.

## Development and CI tools (not linked into shipped code)

| Tool | Licence | Use | Local status |
|---|---|---|---|
| cargo-deny | MIT or Apache-2.0 | Licence and advisory checks | Installed (0.21.0). Not run yet on the workspace |
| cargo-audit | MIT or Apache-2.0 | Advisories | Missing locally |
| cbindgen | MPL-2.0 | Header generation | Installed locally (0.29.4). Not yet used. Used as a tool only, and its output is our own header |
| qpdf | Apache-2.0 | Structure validation (black box) | Installed locally (12.4.2). Not yet used |
| poppler (`pdfinfo`) | GPL | Black-box validation. Run as a separate process only | Installed |
| veraPDF | GPL or MPL | PDF/A and PDF/UA validation, as a separate process | Missing locally |

The licences above are from memory and must be verified when each tool is adopted.

## Source provenance

All code in the repository was written for this project, with AI assistance, from public specifications and general Rust knowledge. No code was copied from a PDF library. The AI-generated code has not yet had independent human review. That review is required (`rules/licensing.md`, item 5).
