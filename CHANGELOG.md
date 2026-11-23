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

### Not included

- PDF serialisation. No PDF file can be produced yet.
