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
- **Resource limits** are constants in the core (page size, page count). Every input-driven allocation gets one.
- **Determinism.** Same input and options give the same bytes. No clock and no random source in the core (ADR-0007).
- **WASM.** The core must build for `wasm32` without threads or a filesystem (ADR-0009). This is not yet verified by CI.
- **Versioning.** SemVer for Rust. A separate `(major, minor)` ABI version for C (ADR-0004).
- **PDF versions.** Default target is PDF 1.7. PDF 2.0 features are opt-in and tied to a version option (ADR-0006).

## Data flow for generation

1. The caller builds a `Document` (through Rust, or through handles over the C ABI).
2. The core validates each input as it is added and rejects values outside the limits.
3. A serialiser (v0.1.0) turns the model into bytes in memory. The caller gets a buffer, or a callback receives chunks. The core does not write files.

## Not decided yet

See `docs/requirements.md` (OQ-03, OQ-05, OQ-06, OQ-09). In particular: supported platforms, compression and font crates, text in v0.1.0, and the timing of the server interfaces.

## Review status

No architecture or security review has been done. See GATE-1 in `docs/roadmap.md`.
