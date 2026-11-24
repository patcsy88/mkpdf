# ADR-0012: Output-buffer ownership across the C ABI

Status: Proposed. Resolves open question OQ-13. Not accepted. Only the project owner accepts it, after the Security Engineer has reviewed this amended text (threat T-23 allocator mismatch, T-24 reentrancy).

**Depends on ADR-0013.** The buffer handle is a `uint64_t` token of kind 2 in the ADR-0013 table (D-13). This ADR cannot be accepted before ADR-0013, and if ADR-0013 changes the token shape, this ADR changes with it. Related: D-02, D-08, D-13, D-14, D-18, D-20, SEC-ADR-05, SEC-ADR-10 to SEC-ADR-13, SEC-ADR-15, RISK-F-P2-04, RISK-F-P2-10, RISK-F-P2-11.

## Changes since first draft

| Change | Reason |
|---|---|
| The sketch uses tokens (`typedef uint64_t MkpdfBuffer;`), not `MkpdfBuffer *`. Accessors and `free` return statuses. The ADR states its dependency on ADR-0013 | SEC-ADR-10, RISK-F-P2-04, D-13. The first draft fixed a pointer shape before the handle ADR existed |
| `mkpdf_buffer_data` and `mkpdf_buffer_len` are replaced by one `mkpdf_buffer_view` | SEC-ADR-11. Two calls can see two different states, and `NULL` did not say why |
| Pointer lifetime rule: valid until `mkpdf_buffer_free` starts. The conflict with D-18 is resolved in the text | SEC-ADR-11 |
| Bytes are stored as an immutable `Arc<[u8]>` held in the table | SEC-ADR-11 |
| `MAX_OUTPUT_BYTES` proposed at `1 << 30`, below `i32::MAX`. A byte budget for live buffers is added. Reservation uses `try_reserve_exact` | SEC-ADR-05, SEC-ADR-12, D-20 |
| "T-23 is closed" reworded to "will be closed when ..." | SEC-ADR-13, RISK-F-P2-10 |
| Option B critique softened | RISK-F-P2-11 |
| `save` is a call on the handle and may return `BUSY` | SEC-ADR-15 |
| Wrapper table updated for tokens and the `view` call | follows from the above |
| A rule amendment for `rules/interoperability.md` item 2 is proposed for the owner | RISK-F-P2-10 |

### Findings not accepted, or accepted only in part

| Finding | Disposition | Reason |
|---|---|---|
| SEC-ADR-12: "the serialiser can count bytes and stop at the limit" | Stated as a requirement on the writer, not as something this ADR can guarantee | The writer does not exist (PDF-001). This ADR sets the contract. Whether growth during serialisation can be made fallible is the writer's design. The text says what remains if it cannot |
| SEC-ADR-12: value `1 << 30` | Proposed, **not decided** | D-20 and the Security review leave the value to the owner. I agree with the reasoning (fits every signed 32-bit length) and propose it, but I cannot set it |
| SEC-ADR-11: store as `Box<[u8]>` | `Arc<[u8]>` instead | ADR-0013 holds every value as an `Arc<T>` so that `free` while a call runs is safe (D-18). A boxed slice inside the `Arc` has the same properties the finding wants: it cannot grow, it does not move, no spare capacity |

## Context

The serialiser (PDF-001, PDF-002) produces a byte string of unknown size. FFI-001 must hand it to the caller. Today the ABI returns no variable-size data. Constraints:

- D-02: the library owns every object it creates and the caller releases it with the matching `*_free`. `rules/interoperability.md` item 2 says the same.
- D-08, D-11, D-13: handles are `uint64_t` tokens, generation-checked, so a double free or use after free returns an error and not undefined behaviour (ADR-0013). A buffer is one more kind in that table.
- The core does no I/O and holds the whole document in memory (ADR-0009). The core returns `Vec<u8>` or a similar owned buffer.
- Memory allocated by Rust's allocator must be released by Rust. Calling `free()` or a managed runtime's deallocator on it is undefined behaviour (threat T-23).
- `rules/security.md` item 2 and `rules/architecture.md` item 6: output size needs a documented limit.
- Callers include Go (CGo), Python, Node.js, .NET, Java and C++ (ADR-0010). Several have GC runtimes, threads that move, and exceptions.

The roadmap text for FFI-001 says "save to a caller buffer or a callback". That wording predates this ADR and is not a decision.

## Options

### A. Library-allocated buffer with a matching free

`mkpdf_document_save(doc, &out)` creates an owned buffer and returns a token for it. `mkpdf_buffer_view` gives a read-only pointer and a length. `mkpdf_buffer_free` releases it.

- For: one call, no size guessing. The same ownership rule as every other object. No allocator mismatch, because the library frees what the library allocated. No layout in the ABI because the buffer is an opaque token. The buffer is immutable after creation, so it may be read from several threads. It fits the generation-checked table with no special case. No call from the library into foreign code, so no reentrancy and no foreign unwinding.
- Against: the whole output exists in memory twice for a moment if the caller copies it into a managed array. The caller must remember to free. Not suited to documents larger than memory (not a goal, see ADR-0009).

### B. Caller-supplied buffer with a size query

`mkpdf_document_save(doc, uint8_t *buf, size_t cap, size_t *out_len)`. The caller calls once with `buf = NULL` to get the size, allocates, and calls again.

- For: the caller picks the allocator, so a managed runtime can serialise straight into its own array (one copy fewer). No library free function. No new object type.
- Against: the library either serialises twice (double CPU cost, and the two results must be byte-identical, which ADR-0007 gives) or caches the first result in the handle (hidden state with a lifetime that the contract must define, and the handle is then no longer logically read-only). Time-of-check to time-of-use: the document may change between the two calls, so the second call can need a different size. Every caller must handle a `cap`-mismatch error path, which is a recurring source of bugs in C APIs. An implementation that honours `cap` cannot overflow the caller's buffer, so safety is not the argument. The argument is double cost, the TOCTOU window and the extra error path. With a cached result the design is option A with extra steps.

### C. Streaming callback

`mkpdf_document_write(doc, sink_fn, user_data)`. The library calls `sink_fn(user_data, ptr, len)` for each chunk.

- For: bounded memory, no large allocation, natural for files and sockets, and a clean fit with a later streaming writer.
- Against: the library calls into foreign code while it holds a reference on the handle. If the callback re-enters the library with the same handle, aliasing rules are broken (SEC-F-11, T-24). A callback that throws (Java, .NET, C++ exceptions) or longjmps unwinds through Rust frames, which is undefined behaviour with `extern "C"` (ADR-0011). A callback that panics in Go via CGo is the same. Callbacks need a function pointer type, a `user_data` convention, a way to report a sink failure and a stop. Each runtime has its own rules and costs: CGo needs an exported Go function and a handle table for `user_data`, JNI needs a global reference and thread attachment, .NET needs a pinned delegate, Node needs thread-safe functions if called on another thread, Python needs the GIL. More surface to test per binding, and more for a hostile or buggy sink to break.

## Decision (proposed)

Use option A: the library allocates, and the caller frees with `mkpdf_buffer_free`. Do not add callbacks now.

Proposed surface (a sketch for review, not yet in the header, and it follows ADR-0013 sections 1, 5 and 7):

```c
typedef uint64_t MkpdfBuffer;   /* token, kind 2. 0 is never valid. Not a pointer */

/* Serialises. On success *out is a new buffer token. On any failure *out is set to 0 first (D-14). */
int32_t mkpdf_document_save(MkpdfDocument doc, MkpdfBuffer *out);

/* One lookup returns both values. On any failure *out_data is set to NULL and *out_len to 0 first. */
int32_t mkpdf_buffer_view(MkpdfBuffer buf, const uint8_t **out_data, size_t *out_len);

/* Releases the buffer. Returns MKPDF_OK, or MKPDF_ERR_INVALID_HANDLE for a stale or foreign token.
   free(0) is a no-op returning MKPDF_OK (D-17). */
int32_t mkpdf_buffer_free(MkpdfBuffer buf);
```

Rules:

1. The buffer is an opaque token, like `MkpdfDocument`. No `{ptr, len}` struct is exposed in the ABI, so no layout is frozen. An invalid token gives `MKPDF_ERR_INVALID_HANDLE` from `view` and from `free`, and the out values are set to `NULL` and `0` before the lookup. The status tells the caller **why** the pointer is `NULL`, which a bare `NULL` could not.
2. `mkpdf_buffer_view` returns a pointer into the buffer and its length from one lookup, so the two always describe the same buffer. The pointer is valid and unchanged **until `mkpdf_buffer_free` starts on that buffer on any thread**. It does not depend on the document, which may be freed first. The data is read-only. Writing through it, while another thread reads it, is a data race in the caller's process, and the header says so.
3. The buffer is immutable and holds its bytes in a fixed `Arc<[u8]>`: it cannot grow, the pointer never moves, no spare capacity is kept. Several threads may call `view` at once and never get `BUSY`.
4. **Free and D-18.** ADR-0013 lets `free` succeed on a handle that a call is using. For a buffer, the pointer from `view` outlives the call, so a `free` on another thread could drop the bytes while the caller still reads them. The rule is: the caller orders its own `free` after its last read of the pointer, in every thread. `free` still succeeds and still returns a status. This is a caller contract that no table can enforce, because the caller holds a raw pointer. It is the one place where memory safety rests on the caller (same class as T-02, not T-03). A wrapper keeps it inside its own object by copying by default.
5. Memory is released only by `mkpdf_buffer_free`. Releasing it with `free()`, `delete` or a runtime allocator is undefined behaviour, and the header says so. With a token handle the caller never holds a pointer to the buffer **object**, so only the data pointer can be freed by mistake.
6. `save` does not change the document and is repeatable. Output for equal input is byte-identical (ADR-0007, deterministic by default, D-12). `save` is still a call on the document handle, so it takes the document lock with `try_lock` and **may return `MKPDF_ERR_BUSY`** if another call is using the document (ADR-0013 section 3, SEC-ADR-15). A wrapper must surface `BUSY` and must not spin.
7. **Output limit.** Output has a documented maximum `MAX_OUTPUT_BYTES`. It is proposed to be **`1 << 30` (1 GiB)**, which fits in every signed 32-bit length (Go `C.int`, JNI `jint`, .NET array length) and so keeps the wrapper conversions free of truncation (SEC-F-01 class). The value is **NOT DECIDED**, and it may only rise by a recorded decision. Past the limit `save` returns `MKPDF_ERR_LIMIT_EXCEEDED`, with `*out = 0`.
8. **Allocation.** The final buffer is reserved with `Vec::try_reserve_exact`, so a failed reservation returns a status and does not abort the host. The writer must stop at `MAX_OUTPUT_BYTES` before it grows past it, so the limit is enforced during serialisation and not after. Whether growth *during* serialisation can be made fallible depends on the writer design (PDF-001). If it cannot, an allocation failure inside the writer still aborts the process, because `catch_unwind` cannot stop it (ADR-0011). The ABI document states which case holds once the writer exists.
9. **Byte budget (D-20).** The buffer table keeps a running total of live buffer bytes, checked under the buffer table lock when a buffer is registered and released when it is freed. A `save` that would pass the budget returns `MKPDF_ERR_LIMIT_EXCEEDED` before it allocates the result where it can. The proposed value is **`4 * MAX_OUTPUT_BYTES` (4 GiB), NOT DECIDED**. Without it the product of the limits is not a bound: 65 536 buffers of 1 GiB would be 64 TiB. The handle count (`MAX_LIVE_HANDLES`, ADR-0013) bounds the table, not memory (SEC-ADR-05). Total document memory is not bounded by the library and is the host's concern.
10. Zero length is not a valid PDF, so a successful `save` never returns an empty buffer.
11. A streaming interface is not ruled out. If memory use matters, a later ADR may add `mkpdf_document_write` as an additive minor-version function. That ADR must settle reentrancy (a busy status), foreign unwinding (ADR-0011) and failure reporting first.

## What it means for each wrapper

Common rule for all wrappers: free the buffer exactly once, make the wrapper's own `Dispose`, `close` or finalizer idempotent (zero the stored token after the free, D-17), and copy by default. Exposing a view into native memory is an opt-in that the wrapper must tie to the buffer's lifetime (rule 4). Convert lengths with an explicit range check. Test a token with `!= 0`, never `> 0`, and carry it as an unsigned 64-bit value (ADR-0013 section 11). Surface `BUSY` and `INVALID_HANDLE` as errors, never retry in a loop.

All behaviour in this table is from knowledge of each runtime. **No wrapper exists and none of it has been run or checked** (SEC review, "Claims about wrapper and language behaviour").

| Wrapper | How the buffer is consumed | Free path | Watch out for |
|---|---|---|---|
| Go (CGo) | `C.mkpdf_buffer_view(buf, &data, &n)`, check the status, then copy with `C.GoBytes(unsafe.Pointer(data), C.int(n))`, then free. Zero-copy `unsafe.Slice` only inside the call that owns the buffer | `defer C.mkpdf_buffer_free(buf)` in the same function. For a long-lived wrapper type, `Close()` plus `runtime.SetFinalizer` as a backstop | With `MAX_OUTPUT_BYTES` at most `1 << 30`, `C.int(n)` cannot overflow, but check `n` anyway. Go memory passed in must not contain Go pointers and must not be kept by C after the call returns. Pin the OS thread (`runtime.LockOSThread`) around a call and the following `mkpdf_last_error_message` read, because goroutines migrate (SEC-F-12). The token is a `uint64` |
| Python | `ctypes` or `cffi`: `ctypes.string_at(ptr, n)` copies into `bytes`. A `memoryview` over native memory only inside a context manager | `Buffer` object with `__enter__`/`__exit__` and `close()`, `__del__` as a backstop. Free after the copy | `__del__` runs at arbitrary times and at interpreter shutdown, when the library may already be unloaded. Declare `restype` and `argtypes` (`c_uint64`, `c_size_t`, `c_void_p`), because `ctypes` defaults to `int`. A PyO3 build (ADR-0010) follows the same rules with `PyBytes::new` as the copy |
| Node.js | N-API: `Buffer.from` copy (`napi_create_buffer_copy`), then free. Zero-copy `napi_create_external_buffer` with a finalizer that calls `mkpdf_buffer_free` only as an opt-in | Finalizer runs on the JS thread at GC time. An explicit `buffer.free()` for deterministic release | The token is a **`BigInt` only**. A `Number` loses precision once the generation reaches 2^21 and can name another live buffer (SEC-ADR-14). External buffers are blocked or restricted in some runtimes (Electron with the V8 memory cage), so the copy is the default |
| .NET | P/Invoke with `ulong` for the token and `nuint` for `size_t`. `Marshal.Copy` into `byte[]`, or a `ReadOnlySpan<byte>` over native memory from a `SafeHandle` subclass | `SafeHandle.ReleaseHandle` calls `mkpdf_buffer_free`. The token is an integer, so `SafeHandle` holds it and an invalid-handle value of 0 | A span must not outlive the `SafeHandle` (use `DangerousAddRef` for the span's lifetime). Async continuations resume on other threads, so read the error text before awaiting (SEC-F-12). `int`-to-`nuint` conversions need a `checked` cast |
| Java | JNI: `NewByteArray` plus `SetByteArrayRegion` copy, then free in the native method. FFM (Java 22 and later): `MemorySegment.reinterpret(len, arena, cleanup)` with the cleanup calling `mkpdf_buffer_free` | JNI: free before returning to Java, so the leak window is inside one native call. FFM: `Arena` close or `Cleaner` | The token is a `long`, negative when bit 63 is set, so test `!= 0`. `NewDirectByteBuffer` over native memory leaves a `ByteBuffer` that outlives the free (use after free), so it is not the default. JNI lengths are `jint`, and `MAX_OUTPUT_BYTES` fits. No Java exception may propagate out of Rust frames. This ADR has no callbacks, which avoids that |
| C++ | RAII wrapper holding a `MkpdfBuffer` token, `std::span<const uint8_t>` view, non-copyable, movable, destructor calls `mkpdf_buffer_free` | Destructor | Exceptions thrown between `save` and the wrapper's construction leak the buffer, so construct the wrapper from the out-parameter immediately |

Why callbacks are not the default for these wrappers: CGo, JNI, .NET and N-API each need extra machinery and have their own failure rules for foreign code called from native code (option C). Option A needs one plain call and one free in each language.

Cost for option A in wrappers that copy: peak memory is about twice the document size for a moment, plus the document itself during `save`. Accepted for now. If a user shows that it matters, a streaming function (rule 11) is the answer, not option B.

## Consequences

- One more handle kind in the ADR-0013 table and three functions (`save`, `view`, `free`). Each needs invalid-token, double-free, limit and byte-budget tests, run under Miri (FFI-002).
- Required tests, none of which exists yet:
  1. `save` then `view` then `free`: the data matches the document bytes, the length matches.
  2. Double free of a buffer returns `INVALID_HANDLE`. `view` on a freed token returns `INVALID_HANDLE` with `NULL` and `0`.
  3. A document token passed to `view`, and a buffer token passed to a document function, return `INVALID_HANDLE`.
  4. `save` at exactly `MAX_OUTPUT_BYTES` succeeds and one byte past returns `LIMIT_EXCEEDED` with `*out = 0`. This needs a test-only way to lower the limit, under `--cfg mkpdf_test_hooks` (ADR-0011), not a Cargo feature.
  5. Byte budget: registering a buffer past the budget returns `LIMIT_EXCEEDED`, and a `free` makes room.
  6. `save` while another call holds the document lock returns `BUSY`.
  7. Under Miri: two threads call `view` on one buffer, and the free runs only after both have joined.
  8. `save` on a poisoned document returns `MKPDF_ERR_INTERNAL` (ADR-0011).
- The C ABI minor version rises when the functions are added. With D-16 the ABI is 0.2 once ADR-0013 lands, and this ADR's functions are additions after that (a further minor, or part of the same release, as the owner decides).
- The ownership table in `docs/api/c-abi.md` gains the buffer rows when the functions exist.
- FFI-001 as worded in the roadmap ("caller buffer or callback") needs re-wording by the Orchestrator once this ADR is accepted. FFI-001 depends on FFI-003.
- **Threat model.** T-23 (allocator mismatch) **will be closed when FFI-003b is implemented and tests 1 to 3 above pass**. It is not closed by this ADR, and today a double free of a buffer would be undefined behaviour like any raw handle (RISK-F-P2-10, SEC-ADR-13). Even then the data pointer remains a caller contract: a caller can still call `free()` on it. T-24 does not arise until a callback exists. T-04 is partly mitigated by the output limit, the byte budget and `try_reserve_exact`, with the writer caveat in rule 8.
- **Rules amendment proposed for the owner (not made here).** `rules/interoperability.md` item 2 says a borrowed pointer is valid "only until the next call on the same thread". The `view` pointer is valid until `mkpdf_buffer_free` starts on that buffer. Proposed wording: "or until the owning handle is freed, as documented per function." The rules file needs the owner's approval, so I have not edited it.
- `save` after a handle was poisoned by a panic (ADR-0011) returns `MKPDF_ERR_INTERNAL`.

## Still open (NOT DECIDED)

| # | Point | State |
|---|---|---|
| 1 | Acceptance of this ADR | The owner, after ADR-0013 is accepted and the Security Engineer has reviewed this text |
| 2 | Value of `MAX_OUTPUT_BYTES` | Proposed `1 << 30`. Needed before FFI-001. The owner sets it (D-20 asks for a byte budget, and the Security Engineer proposed this value) |
| 3 | Value of the live-buffer byte budget | Proposed `4 * MAX_OUTPUT_BYTES`. Owner sets it, with the writer |
| 4 | A copy-out function for managed runtimes (`mkpdf_buffer_copy(buf, dst, cap, &n)`) | Not now. Revisit if a wrapper shows the second copy matters |

## Not examined

- No code was written or run. Every statement about wrapper and language behaviour is from knowledge and is unverified.
- `try_reserve_exact`, `Arc<[u8]>` behaviour and Miri support for the threaded test are from knowledge of the standard library and are not run here.
- Whether the writer can enforce the limit incrementally is unknown, because no writer exists.
- The Security Engineer has not reviewed this amended text.
