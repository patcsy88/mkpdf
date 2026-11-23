# ADR-0003: Opaque-handle C ABI

Status: Proposed. Follows owner decision D-02.

## Context
Many languages must call the core. Rust structs have no stable layout, and Rust panics must not unwind into foreign code.

## Decision
- Objects are opaque pointers created and freed by the library. The library owns them, and callers free them once with the matching `*_free`. `free(NULL)` is a no-op.
- Functions return an `i32` status (`0` = success). Results come back through out-parameters.
- The text of the last error is kept per thread and read with `mkpdf_last_error_message`. It is valid until the next call on that thread, and a success clears it.
- Strings are UTF-8, as pointer plus length.
- Each function is wrapped in `catch_unwind`. A panic returns `MKPDF_ERR_INTERNAL`.
- A handle is used by one thread at a time. Distinct handles may be used concurrently.

## Alternatives
- Serialised document description (JSON or binary) in one call: simpler, but poor for streaming and fine control. Not chosen by the owner.
- Caller-owned structs: they lock the layout into the ABI.

## Consequences
Every binding needs a small wrapper that frees handles (finalizers, `Dispose`, RAII). Double free and use after free are undefined behaviour in C. The safety rests on the contract and on tests. Debug-time handle validation (for example a magic tag) is an open option for GATE-1.
