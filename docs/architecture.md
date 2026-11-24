# Architecture (ARCH-001)

Status: draft. It becomes approved when GATE-1 passes and the review is recorded in `docs/reviews/`.

## Goals and constraints

One Rust engine, consumed by many languages through a stable C ABI. Safe Rust by default, offline, deterministic where supported, and testable. The brief fixes the licence (Apache-2.0), clean-room development and the interface list.

## Components

```
            +-----------+  +-----------+  +--------------------+
 Rust apps  |  mkpdf-cli|  | SDKs      |  | future: REST, gRPC |
            +-----+-----+  | C++ Py    |  +---------+----------+
                  |        | Node .NET |            |
                  |        | Java Go   |            |
                  |        +-----+-----+            |
                  |              | C ABI            |
                  |        +-----v-----+            |
                  |        | mkpdf-ffi |            |
                  |        | (unsafe)  |            |
                  |        +-----+-----+            |
                  +--------------+-------------------+
                                 |
                          +------v------+
                          | mkpdf-core  |  safe Rust, no I/O,
                          |             |  no threads, no globals
                          +-------------+
```

| Component | Responsibility | Must not |
|---|---|---|
| `mkpdf-core` | Document model, layout (later), PDF writer and parser (later) | Do I/O, spawn threads, read the clock, use `unsafe` |
| `mkpdf-ffi` | C ABI: opaque handles, status codes, panic guard, thread-local error text | Contain PDF logic. It only translates |
| `mkpdf-cli` | Command line over the core | Contain PDF logic |
| SDKs | Idiomatic wrappers over the C ABI | Reimplement PDF behaviour |
| Servers (REST, gRPC) | Network access to the core | Be linked into the core |

Planned internal modules of `mkpdf-core`, split into separate crates only when a second consumer needs it:

| Module | Purpose | Release |
|---|---|---|
| `model` | Document, pages, metadata | v0.1.0 (partial) |
| `pdf::object`, `pdf::writer` | Object model, serialiser, cross-reference | v0.1.0 |
| `content` | Content streams, graphics operators | v0.1.0 |
| `font`, `text` | Font embedding, Unicode | v0.3.0 (see SCOPE-01) |
| `layout` | Paragraphs, tables, pagination | v0.3.0 |
| `parse` | Parser (hostile input) | v0.4.0 |
| `crypto` | Encryption, signatures | v0.5.0 |

## Key properties

- **Layering is enforced by the compiler.** The core forbids `unsafe`. Only the FFI crate allows it.
- **Resource limits** are constants in the core (page size, page count, title length). The rule is that every input-driven allocation gets one (`rules/architecture.md` item 6). That is the target, not yet a fact: output size, handle count and live buffer bytes are proposed in ADR-0012 and ADR-0013 and do not exist in code.
- **Determinism.** Same input and options give the same bytes. No clock and no random source in the core (ADR-0007).
- **WASM.** The core must build for `wasm32` without threads or a filesystem (ADR-0009). A CI job for this (`wasm-core`) builds `mkpdf-core` for `wasm32-unknown-unknown` and passed in CI run 45103722429. It runs no test on wasm, so "supported" is not yet shown (OQ-09). On `wasm32-unknown-unknown` a panic aborts, so the FFI panic guard (ADR-0011) does not apply there.
- **Versioning.** SemVer for Rust. A separate `(major, minor)` ABI version for C (ADR-0004).
- **PDF versions.** Default target is PDF 1.7. PDF 2.0 features are opt-in and tied to a version option (ADR-0006).

## Data flow for generation

1. The caller builds a `Document` (through Rust, or through handles over the C ABI).
2. The core validates each input as it is added and rejects values outside the limits.
3. A serialiser (v0.1.0) turns the model into bytes in memory. The core does not write files. How the bytes reach a C caller is not decided: ADR-0012 (Proposed) recommends a library-allocated buffer that the caller releases with `mkpdf_buffer_free`. It compares this with a caller-supplied buffer and a streaming callback, and proposes no callback for now. Until the owner decides, no serialisation function exists in the C ABI.

## Not decided yet

See `docs/requirements.md` (OQ-03, OQ-05, OQ-06, OQ-09 to OQ-13). In particular:

- Supported platforms are decided (OQ-03) but only macOS arm64 has been tested, by hand. Compression and font crates (OQ-05) are not chosen.
- Determinism defaults, the document ID and dates (OQ-10, ADR-0007).
- Panic strategy: ADR-0011 is Proposed (OQ-12).
- Output-buffer ownership: ADR-0012 is Proposed (OQ-13).
- Generation-checked handles (D-08, FFI-003). The owner has decided to build them. The ADR that amends ADR-0003 and fixes the ABI shape is not written yet.
- The ABI version rule while the major is 0 (`docs/reviews/gate-1-architecture.md`, ARCH-F-03).
- A record of resource limits and their tests (ARCH-F-07).

## Review status

GATE-1 review: the architecture review is recorded in `docs/reviews/gate-1-architecture.md` (pass with conditions, single-role, not independent) and the security review in `docs/reviews/gate-1-security.md`. This document stays a draft. The conditions in those records are open, and no ADR is Accepted.
