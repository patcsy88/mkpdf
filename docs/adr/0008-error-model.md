# ADR-0008: Error model

Status: Proposed

## Decision
- The core returns `Result<_, Error>`. `Error` has a `#[non_exhaustive]` `ErrorKind` and a message. Callers match on the kind. The message is not a contract.
- The library never panics on bad input.
- The FFI maps kinds to fixed status codes (`docs/api/c-abi.md`). Unknown future kinds map to `MKPDF_ERR_INVALID_ARGUMENT` until they get a code of their own.
- Messages must not include input content that could be sensitive.

## Consequences
Adding a kind is compatible for Rust callers (non-exhaustive) and needs a deliberate ABI decision.
