# ADR-0012: Output-buffer ownership across the C ABI

Status: Proposed. Resolves open question OQ-13. Not accepted. Only the project owner accepts it, after the Security Engineer has reviewed it (threat T-23 allocator mismatch, T-24 reentrancy).

## Context

The serialiser (PDF-001, PDF-002) produces a byte string of unknown size. FFI-001 must hand it to the caller. Today the ABI returns no variable-size data. Constraints:

- D-02: the library owns every object it creates and the caller releases it with the matching `*_free`. `rules/interoperability.md` item 2 says the same.
- D-08: handles are generation-checked, so a double free or use after free returns an error and not undefined behaviour (ADR for that change still to be written, backlog FFI-003, which must land before FFI-001). Any buffer design must fit in the same scheme.
- The core does no I/O and holds the whole document in memory (ADR-0009). The core returns `Vec<u8>` or a similar owned buffer.
- Memory allocated by Rust's allocator must be released by Rust. Calling `free()` or a managed runtime's deallocator on it is undefined behaviour (threat T-23).
- `rules/security.md` item 2 and `rules/architecture.md` item 6: output size needs a documented limit.
- Callers include Go (CGo), Python, Node.js, .NET, Java and C++ (ADR-0010). Several have GC runtimes, threads that move, and exceptions.

The roadmap text for FFI-001 says "save to a caller buffer or a callback". That wording predates this ADR and is not a decision.

## Options

### A. Library-allocated buffer with a matching free

`mkpdf_document_save(const MkpdfDocument *doc, MkpdfBuffer **out)` creates an owned opaque buffer. Accessors give a read-only pointer and a length. `mkpdf_buffer_free` releases it.

- For: one call, no size guessing. The same ownership rule as every other object. No allocator mismatch, because the library frees what the library allocated. No layout in the ABI when the buffer is an opaque handle. The buffer is immutable after creation, so it may be read from several threads. Fits generation-checked handles unchanged. No call from the library into foreign code, so no reentrancy and no foreign unwinding.
- Against: the whole output exists in memory twice for a moment if the caller copies it into a managed array (library copy plus the managed copy). The caller must remember to free. Not suited to documents larger than memory (not a goal, see ADR-0009).

### B. Caller-supplied buffer with a size query

`mkpdf_document_save(doc, uint8_t *buf, size_t cap, size_t *out_len)`. The caller calls once with `buf = NULL` to get the size, allocates, and calls again.

- For: the caller picks the allocator, so a managed runtime can serialise straight into its own array (one copy fewer). No library free function. No new object type.
- Against: the library either serialises twice (double CPU cost, and the two results must be byte-identical, which ADR-0007 gives but a bug would turn into a heap overflow in the caller's buffer) or caches the first result in the handle (hidden state with a lifetime that the contract must define, and the handle is then no longer logically read-only). Time-of-check to time-of-use: the document may change between the two calls, and the second call then has a different size. The size-mismatch path is a recurring source of overflow bugs in C APIs. Callers get `cap` wrong. With a cached result the design is option A with extra steps.

### C. Streaming callback

`mkpdf_document_write(doc, sink_fn, user_data)`. The library calls `sink_fn(user_data, ptr, len)` for each chunk.

- For: bounded memory, no large allocation, natural for files and sockets, and a clean fit with a later streaming writer.
- Against: the library calls into foreign code while it holds a `&mut` or `&` on the handle. If the callback re-enters the library with the same handle, aliasing rules are broken (SEC-F-11, T-24). A callback that throws (Java, .NET, C++ exceptions) or longjmps unwinds through Rust frames, which is undefined behaviour with `extern "C"` (ADR-0011). A callback that panics in Go via CGo is the same. Callbacks need a function pointer type, a `user_data` convention, a way to report a sink failure and a stop. Each runtime has its own rules and costs: CGo needs an exported Go function and a handle table for `user_data` (Go pointers must not be kept by C), JNI needs a global reference and thread attachment, .NET needs a pinned delegate (`GCHandle`) so the GC does not move or collect it, Node needs thread-safe functions if called on another thread, Python needs the GIL. More surface to test per binding, and more for a hostile or buggy sink to break.

## Decision (proposed)

Use option A: the library allocates, and the caller frees with `mkpdf_buffer_free`. Do not add callbacks now.

Proposed surface (a sketch for review, not yet in the header):

```c
typedef struct MkpdfBuffer MkpdfBuffer;   /* opaque, owned by the library */

/* Serialises. On success *out is a new buffer. On any failure *out is set to NULL first. */
int32_t        mkpdf_document_save(const MkpdfDocument *doc, MkpdfBuffer **out);
const uint8_t *mkpdf_buffer_data(const MkpdfBuffer *buf);  /* NULL if buf is invalid */
size_t         mkpdf_buffer_len(const MkpdfBuffer *buf);   /* 0 if buf is invalid */
void           mkpdf_buffer_free(MkpdfBuffer *buf);        /* NULL is a no-op */
```

Rules:

1. The buffer is an opaque handle, like `MkpdfDocument`. No `{ptr, len}` struct is exposed in the ABI, so no layout is frozen. It is generation-checked under D-08, and a stale or foreign buffer pointer returns `NULL` or `0` from the accessors and is a no-op in `free`.
2. `mkpdf_buffer_data` returns a pointer into the buffer. It stays valid and unchanged until `mkpdf_buffer_free` on that buffer. It does not depend on the document, which may be freed first. The data is read-only for the caller.
3. The buffer is immutable. Several threads may read it at once. `free` must not run concurrently with any other call on the same buffer.
4. Memory is released only by `mkpdf_buffer_free`. Releasing it with `free()`, `delete` or a runtime allocator is undefined behaviour and the header says so.
5. `save` does not change the document. It takes `const` and is repeatable. Output for equal input is byte-identical (ADR-0007).
6. Output has a documented maximum (`MAX_OUTPUT_BYTES`, value to be set with the writer, covered by a boundary test) and returns `MKPDF_ERR_LIMIT_EXCEEDED` past it. Allocation failure aborts the process in Rust (`catch_unwind` cannot stop it). The limit keeps that outcome far from any normal input. The ABI doc states this.
7. Zero length is not a valid PDF, so a successful `save` never returns an empty buffer. The accessors still define `len = 0` for an invalid handle.
8. A streaming interface is not ruled out. If memory use matters, a later ADR may add `mkpdf_document_write` as an additive minor-version function. That ADR must settle reentrancy (a busy flag that returns an error), foreign unwinding (ADR-0011) and failure reporting before it is accepted.

## What it means for each wrapper

Common rule for all wrappers: free the buffer exactly once, make the wrapper's own `Dispose`, `close` or finalizer idempotent, and copy by default. Exposing a view into native memory is an opt-in that the wrapper must tie to the buffer's lifetime. Convert lengths with an explicit range check (`size_t` is unsigned, and a negative managed length turned into `size_t` is the failure described in SEC-F-01).

| Wrapper | How the buffer is consumed | Free path | Watch out for |
|---|---|---|---|
| Go (CGo) | `C.mkpdf_buffer_data` and `C.mkpdf_buffer_len`, then `C.GoBytes(ptr, C.int(n))` to copy, then free. Zero-copy `unsafe.Slice` only inside the call that owns the buffer | `defer C.mkpdf_buffer_free(buf)` in the same function. For a long-lived wrapper type, `Close()` plus `runtime.SetFinalizer` as a backstop | `C.int` conversion overflows above 2 GiB, so check `n` or use `unsafe.Slice` and copy with `copy`. Go memory passed in (strings, byte slices) must not contain Go pointers and must not be kept by C after the call returns. Pin the OS thread (`runtime.LockOSThread`) around a call and the following `mkpdf_last_error_message` read, because goroutines migrate (SEC-F-12) |
| Python | `ctypes` or `cffi`: `ctypes.string_at(ptr, n)` copies into `bytes`. A `memoryview` over native memory only inside a context manager | `Buffer` object with `__enter__`/`__exit__` and `close()`, `__del__` as a backstop. Free after the copy | `__del__` runs at arbitrary times and at interpreter shutdown, when the library may already be unloaded. `ctypes` defaults to `int` returns, so declare `restype` and `argtypes` (`c_size_t`, `c_void_p`). A PyO3 build (ADR-0010) follows the same rules with `PyBytes::new` as the copy |
| Node.js | N-API: `Buffer.from` copy (`napi_create_buffer_copy`), then free. Zero-copy `napi_create_external_buffer` with a finalizer that calls `mkpdf_buffer_free` only as an opt-in | Finalizer runs on the JS thread, at GC time. An explicit `buffer.free()` for deterministic release | External buffers are blocked or restricted in some runtimes (Electron with the V8 memory cage), so the copy is the default. Do not call the library from a worker thread with a handle created on another one. Pass `BigInt` or checked `Number` for lengths, not a negative value |
| .NET | P/Invoke with `nuint` for `size_t`. `Marshal.Copy` into `byte[]`, or expose `ReadOnlySpan<byte>` over native memory from a `SafeHandle` subclass | `SafeHandle.ReleaseHandle` calls `mkpdf_buffer_free`. `IDisposable` plus finalizer through `SafeHandle` makes double free harmless in managed code | A span must not outlive the `SafeHandle` (use `DangerousAddRef` for the span's lifetime). Async continuations resume on other threads, so read the error text before awaiting (SEC-F-12). `int`-to-`nuint` conversions need a `checked` cast |
| Java | JNI: `NewByteArray` plus `SetByteArrayRegion` copy, then free in the native method. FFM (Java 22 and later): `MemorySegment.reinterpret(len, arena, cleanup)` with the cleanup calling `mkpdf_buffer_free` | JNI: free before returning to Java, so the leak window is inside one native call. FFM: `Arena` close or `Cleaner` | `NewDirectByteBuffer` over native memory leaves a `ByteBuffer` that outlives the free (use after free), so it is not the default. JNI lengths are `jint` (signed), so check for negative values and for sizes above `jint` max. No Java exception may propagate out of Rust frames. This ADR has no callbacks, which avoids that |
| C++ | RAII wrapper holding `MkpdfBuffer*`, `std::span<const uint8_t>` view, non-copyable, movable, destructor calls `mkpdf_buffer_free` | Destructor | Exceptions thrown between `save` and the wrapper's construction leak the buffer, so construct the wrapper from the out-parameter immediately |

Why callbacks are not the default for these wrappers: CGo, JNI, .NET and N-API each need extra machinery and have their own failure rules for foreign code called from native code (see option C). Option A needs one plain call and one free in each language.

Cost for option A in wrappers that copy: peak memory is about twice the document size for a moment. Accepted for now. If a user shows that it matters, a streaming function (rule 8) is the answer, not option B.

## Consequences

- One more handle type and four functions. Each needs null, stale-handle, double-free and limit tests, run under Miri (FFI-002).
- The C ABI minor version rises when the functions are added (ADR-0004, `rules/interoperability.md` item 7).
- The ownership table in `docs/api/c-abi.md` gains the buffer rows when the functions exist.
- FFI-001 as worded in the roadmap ("caller buffer or callback") needs to be re-worded by the Orchestrator once this ADR is accepted, and FFI-001 should depend on FFI-003.
- Threat model: T-23 is closed by rules 1, 2 and 4. T-24 does not arise until a callback exists.
- `save` after a handle was poisoned by a panic (ADR-0011) returns `MKPDF_ERR_INTERNAL`.

## Open

- Value of `MAX_OUTPUT_BYTES`.
- Whether `mkpdf_buffer_data` should return a non-null pointer for an empty buffer. Proposed: never needed, see rule 7.
- Whether the buffer type should also carry a copy-out function (`mkpdf_buffer_copy(buf, dst, cap, &n)`) for managed runtimes that want to avoid a second native-to-managed copy. Proposed: not now.
