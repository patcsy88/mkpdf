# ADR-0004: ABI and API versioning

Status: Proposed

## Decision
- The Rust API follows SemVer. While the version is 0.x, minor releases may break it.
- The C ABI has its own `(major, minor)`, returned by `mkpdf_abi_version()` as `(major << 16) | minor`. New functions raise the minor version. A change to or removal of a function, a status code value or a documented behaviour raises the major version.
- Status code values never change after release.
- Bindings check the major version at load time and refuse a mismatch.
- The ABI is declared stable only at v1.0.0. Until then it is `0.x` and may change, and the docs say so.

## Consequences
The ABI version is tracked apart from the crate version. A table in `docs/api/c-abi.md` links them.
