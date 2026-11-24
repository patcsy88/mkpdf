# Changelog

All notable changes are recorded here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Cargo workspace with `mkpdf-core`, `mkpdf-ffi` and `mkpdf-cli`.
- Document model: `Document`, `Page`, `Error` with validated page sizes and a page limit.
- C ABI skeleton: version functions, document lifecycle, title, add page, page count, thread-local error message, panic guard.
- Engineering rules, governance files, ADRs, architecture, threat model, API specifications, roadmap.
- Agent definitions for OpenCode.
- CI workflow definitions and `deny.toml`.

### Changed

- `Document::set_title` takes `impl AsRef<str>` and returns `Result<(), Error>`. Titles are limited to `MAX_TITLE_BYTES` (4096) bytes, and a longer title returns `LimitExceeded`. Callers that passed an owned `String` must pass a reference.

### Fixed

- `mkpdf_document_set_title` rejects a length above `isize::MAX` before building a slice (SEC-F-01) and a length above the title limit (SEC-F-02).

### Not included

- PDF serialisation. No PDF file can be produced yet.
