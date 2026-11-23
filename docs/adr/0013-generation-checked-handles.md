# ADR-0013: Generation-checked handles

Status: Proposed. Not accepted. Only the project owner accepts it, after the Security Engineer has reviewed it. Implements owner decisions D-08 and D-11. Backlog item FFI-003. Amends ADR-0003 (see "What this amends"). Related: ADR-0004, ADR-0011, ADR-0012, ARCH-F-01, ARCH-F-03, FFI-CR-03 to FFI-CR-05, SEC-F-04, threat model T-03.

Decisions below that are marked **NOT DECIDED** are recommendations for the owner. They are not settled. Everything else in "Decision (proposed)" is proposed as a whole and is also not accepted.

## Context

- D-08 (owner): a double free, a use after free and a foreign pointer must return an error and never cause undefined behaviour. The ABI is 0.x, so the change is allowed now.
- D-11 (owner): the mechanism is table indices with a generation counter, so a forged or stale handle is never dereferenced. The architect writes the ADR before FFI-001.
- Today (`crates/mkpdf-ffi/src/lib.rs` at `a57d899`) a handle is a `Box::into_raw` pointer to `MkpdfDocument` (lines 36-39, 158). A pointer cannot be checked without reading through it, and reading freed or foreign memory is itself undefined behaviour (SEC-F-04, FFI-CR-04). So no check can be added to the current design. The handle has to stop being something the library dereferences.
- The one-thread-at-a-time rule for a handle is only a written contract today (`docs/api/c-abi.md`, ADR-0003). Nothing detects a violation.
- Eight non-test `unsafe` blocks exist today (FFI-CR-15). Five of them exist only because a raw handle is turned into a reference (`free`, `set_title`, `add_page`, `page_count` and the `Box::from_raw`).
- ADR-0012 proposes a second handle type, the output buffer. ADR-0011 proposes a poison rule after a caught panic. This ADR has to serve both.

## What this amends in ADR-0003

ADR-0003 is Proposed and is not edited by this change. If the owner accepts this ADR, ADR-0003 should be marked Superseded in part, or amended in the same commit, as follows:

| ADR-0003 text | Change |
|---|---|
| "Objects are opaque pointers created and freed by the library" | Objects are referred to by opaque 64-bit tokens. The library still owns them, and the caller releases each with the matching `*_free`. |
| "`free(NULL)` is a no-op" | `free(0)` is the equivalent. See open point 5. |
| "A handle is used by one thread at a time" | Unchanged as a rule for the caller. It is now also enforced: a second concurrent call gets a busy status instead of a data race. |
| Consequences: "Double free and use after free are undefined behaviour in C" | No longer true for handles. They return an error status. Memory safety no longer rests on the caller contract for handle validity. It still rests on it for the other pointers (see "Unsafe blocks"). |
| Consequences: "Debug-time handle validation (for example a magic tag) is an open option for GATE-1" | Closed. A magic tag inside the object is rejected (alternative B). |
| Unchanged | Status codes as the return value, per-thread error text, UTF-8 pointer-plus-length strings, panics caught at the boundary (ADR-0011), library-owned objects. |

## Decision (proposed)

### 1. Token type and layout

A handle is a `uint64_t` value. It is never a pointer and is never dereferenced.

```
bit 63            32 31      24 23                 0
    | generation    |  kind    |       index       |
```

- `index` (24 bits): the slot number in the table for that kind.
- `kind` (8 bits): which table. Starts at 1 for documents. ADR-0012 buffers would use 2. Kind 0 is invalid. The kind is needed so that a document token passed to a buffer function is rejected, and not accepted because the numbers happen to line up. This refines the sketch in FFI-CR-05, which had no kind field.
- `generation` (32 bits): starts at 1 for every slot and rises each time the slot is freed. Because generation is never 0, the token `0` is never valid. It is the "no handle" value.
- 2^24 slots per kind is far above any sensible `MAX_LIVE_HANDLES` (see below).
- The C type is `typedef uint64_t MkpdfDocument;`. The function signatures take `MkpdfDocument doc` by value, and `new` takes `MkpdfDocument *out`. Whether the typedef keeps its old name is a naming point for the header, not a design point.

### 2. Table structure and locking

One table per kind, in the FFI crate only. `mkpdf-core` is untouched (no global state in core, `rules/architecture.md` item 2).

```rust
struct Slot<T> { generation: u32, value: Option<Arc<T>> }
struct Table<T> { slots: Vec<Slot<T>>, free: Vec<u32>, live: usize }
static DOCS: Mutex<Table<Mutex<Document>>> = Mutex::new(Table::new());
const MAX_LIVE_HANDLES: usize = /* value NOT DECIDED, boundary-tested; at most 1 << 24 */;
```

- `new`: lock the table, take an index from `free` or push a slot, store `Arc::new(Mutex::new(Document::new()))`, set `live += 1`, build the token. Past `MAX_LIVE_HANDLES` it returns `MKPDF_ERR_LIMIT_EXCEEDED` and writes `0` to the out token.
- Lookup (used by every function except `new`): lock the table, split the token, check `kind`, check `index < slots.len()`, check `slot.generation == token.generation`, check `value` is `Some`, clone the `Arc`, release the table lock. Any failed check returns `MKPDF_ERR_INVALID_HANDLE`. A forged value fails on the index, the kind or the generation, and no memory is read through it.
- The table lock is held only for this index arithmetic and one `Arc` clone. No document code, no allocation of a document and no drop of a document runs under it. So there is one lock order and no deadlock: the table lock is never held while a document lock is taken.
- A panic while the table lock is held is not expected (the operations are index arithmetic and `Vec::push` bounded by the limit). If the table mutex is poisoned anyway, recover with `PoisonError::into_inner` and carry on. The table's invariants hold after every complete step, so this is safe. The workspace lints deny `unwrap` and `expect` but not this call.
- The table is a global `static` with a lock, so every call from every thread takes one short global lock. This costs some throughput under heavy multi-threaded use. Sharding or a read-write lock is a later change that needs no ABI change.

### 3. Concurrency: busy status, not blocking

After lookup, the call does `try_lock` on the document's own mutex.

- If the lock is free, the call runs.
- If another call holds it, the call returns `MKPDF_ERR_BUSY` at once. It does not wait.
- Recommendation and reason: the contract is "one thread at a time per handle". A caller that breaks it has a bug, and blocking would hide the bug, turn it into a latency problem and can deadlock if a wrapper holds its own lock across the call. A busy status makes the bug visible and testable. The caller may retry.
- Distinct handles never contend on a document lock. They share only the short table lock.
- A handle may be used from different threads one after another. No thread affinity exists.
- Call on a handle while `free` runs on it: `free` takes the slot out of the table first, so later lookups fail with `INVALID_HANDLE`. A call that already cloned the `Arc` finishes on a valid object, and the document is dropped when the last clone is dropped. So there is no use after free in either order.
- Blocking is the alternative if the owner prefers it. See open point 3.

### 4. Poisoning after a panic (link to ADR-0011)

- A function runs its work inside `guard` (`catch_unwind`) and holds the document mutex guard inside that closure. If the closure panics, unwinding drops the guard while panicking, and the standard `Mutex` marks itself poisoned. No extra flag or code is needed. This is the "poisoned flag" in ADR-0011 item 4 option (b), obtained from the standard library.
- Every later call on that handle sees a poisoned lock and returns `MKPDF_ERR_INTERNAL`. `free` still works on a poisoned handle, so the caller can always release it.
- This depends on `panic = "unwind"` (ADR-0011). Under `abort` there is nothing to poison, which is one more reason for the `compile_error!` proposed there.
- A panic in a function that holds no document lock (for example `new`) leaves nothing to poison. The table was not changed in a half-finished way, because it is updated only after the fallible work is done.
- It does not protect against a panic that happens after the guard is released. Code in a function should keep all document mutation inside the locked section.

### 5. Free semantics

`mkpdf_document_free(MkpdfDocument doc)` now returns `int32_t`.

- Valid token: remove the value from the slot, bump the generation, push the index on the free list, release the table lock, then drop the `Arc`. Returns `MKPDF_OK`.
- Slot reuse: if the generation would pass `u32::MAX`, the slot is retired and never reused (it is not put on the free list). A retired slot costs one slot of memory for good. Reaching this takes about four billion frees of one slot, so it is a safety rule, not a practical cost. It is a unit test target.
- Stale, already freed, foreign or wrong-kind token: returns `MKPDF_ERR_INVALID_HANDLE`. No memory is touched.
- Token `0`: see open point 5. Recommendation: a no-op returning `MKPDF_OK`.
- Free works on a poisoned or busy document. If the document is busy on another thread, free still removes it from the table and the running call completes on its own clone.
- Two threads freeing the same token at once: one gets `OK`, the other `INVALID_HANDLE`.
- Because `free` takes only a token, it needs no `unsafe` and can be a safe `extern "C" fn`.

### 6. New status codes

**NOT DECIDED.** Proposed, additive, values never change once released:

| Value | Name | Meaning |
|---|---|---|
| 5 | `MKPDF_ERR_INVALID_HANDLE` | The token is zero (where zero is not allowed), stale, foreign, or of the wrong kind |
| 6 | `MKPDF_ERR_BUSY` | The handle is in use by another call. The caller broke the one-thread-at-a-time rule, and may retry |

The existing `MKPDF_ERR_NULL_POINTER` stays for out parameters and strings. For a handle argument it no longer applies, because a handle is not a pointer. `MKPDF_ERR_INTERNAL` (100) also covers a poisoned handle. The error text distinguishes the cases. The header, `docs/api/c-abi.md`, the Rust constants and the SDK status tables all gain the two rows. A test compares them (FFI-CR-09).

### 7. Effect on ADR-0012 (output buffers)

- `MkpdfBuffer` becomes a token of kind 2 in a second table, `Table<Vec<u8>>`. A buffer is immutable, so it needs no per-object mutex and never returns `BUSY`. Many threads may read it.
- `mkpdf_buffer_data` and `mkpdf_buffer_len` return `NULL` and `0` for an invalid token, which matches the rules in ADR-0012. `mkpdf_buffer_free` now returns a status like document `free`.
- The `data` pointer is a real pointer into the `Vec`, and it stays valid until `mkpdf_buffer_free`. That is a caller contract that no table can enforce, because the caller holds a raw pointer. It is the one place where memory safety still rests on the caller after this ADR. A wrapper copies by default (ADR-0012) so this contract stays inside the wrapper.
- A generic `Table<T>` is justified because two uses then exist (documents and buffers, `rules/architecture.md`: no abstraction without a second use). If ADR-0012 is not accepted, the table stays a plain document table.
- The out-parameter rule applies to buffers too: `mkpdf_document_save` writes `0` to the out token first.

### 8. ABI version impact

- Every function that takes or returns a handle changes: `new`, `free`, `set_title`, `add_page`, `page_count` (and everything added later). The header changes. This is an incompatible ABI change.
- ADR-0004 and `rules/interoperability.md` item 7 say a breaking change raises the major, and ADR-0004 also says the ABI is "0.x and may change" until 1.0. Read literally, this change makes the ABI 1.0, which contradicts "frozen at v1.0.0". This is ARCH-F-03.
- **NOT DECIDED.** Recommendation: while the major is 0, any change that is not a pure addition raises the minor, and bindings accept only an exact major-and-minor match. This change moves the ABI from 0.1 to **0.2**. ADR-0004 and `rules/interoperability.md` item 7 need the matching amendment, which is the owner's to approve. See open point 4.
- Nothing has been released, so no installed user is affected. The cost lands on the wrappers that do not exist yet.

### 9. What happens to the unsafe blocks

Today: 8 non-test `unsafe` blocks (FFI-CR-15). After this ADR, with the helpers proposed in FFI-CR-03:

| Block today | After |
|---|---|
| `*out = handle` in `new` | One shared `write_out` helper (1 block for all out writes) |
| `Box::from_raw` in `free` | Gone. `free` uses only the table |
| `doc.as_mut()` in `set_title` | Gone. The `&mut Document` comes from a `MutexGuard` |
| `from_raw_parts` in `set_title` | Stays (1 block). The string pointer is still a pointer |
| `doc.as_mut()` in `add_page` | Gone |
| `*out_index = ...` in `add_page` | `write_out` |
| `doc.as_ref()` in `page_count` | Gone |
| `*out_count = ...` in `page_count` | `write_out` |

Result: **2** non-test `unsafe` blocks (`write_out` and `from_raw_parts`), down from 8. Without the `write_out` helper it is 4. These are counts of blocks from a design, not of code that exists. The remaining blocks are exactly the ones that handle caller-supplied raw pointers, which no table can check. What the change removes is the class of bug where a stale handle becomes a dangling reference.

- Functions that take only tokens (`free`, `page_count` apart from its out pointer) may no longer need to be `unsafe extern "C" fn`. Whether a function that takes a raw out pointer must stay `unsafe` is a Rust-side choice that the FFI crate decides.
- The `#![allow(unsafe_code)]` can move from the crate to the two helpers and the exports (check first that the `#[unsafe(no_mangle)]` attribute does not need the allow, FFI-CR-13).

### 10. Required tests

All in the FFI crate and run under Miri. None exists yet.

1. Lifecycle: new, use, free, with the token checked non-zero and kind and generation as designed.
2. Double free: second `free` returns `INVALID_HANDLE`. Document and buffer.
3. Use after free: every function on a freed token returns `INVALID_HANDLE`, for every function that takes a handle.
4. Slot reuse: after free and new, the new token differs from the old one (the generation changed), and the old token is still rejected while the new one works.
5. Forged tokens: zero, `u64::MAX`, an index beyond the table, a valid index with the wrong generation (plus and minus one), kind 0, an unknown kind, and the token of the other kind. A property test over random `u64` values never panics and never returns `OK` for a token that was not issued.
6. Generation retirement: a table test with the generation near `u32::MAX` shows the slot is retired and not reused. Needs a test-only constructor, not a public hook.
7. Limit: with a small `MAX_LIVE_HANDLES` in a test table, the one-past request returns `LIMIT_EXCEEDED`, writes `0` to the out token, and a free makes room again.
8. Busy: a test holds the document lock and calls an export, which must return `BUSY` and not block. After release the call works.
9. Concurrency: several threads each use their own handle, and several threads race to free one token (exactly one `OK`). Run both under Miri with threads.
10. Free while a call is running: the running call completes, then the document is dropped, with no use after free (Miri).
11. Poison: a test-only panic inside a locked call (behind a feature never built in release, as in ADR-0011) makes later calls return `MKPDF_ERR_INTERNAL`, and `free` still returns `OK`.
12. Out parameters: each function writes the documented neutral value on every failure path (open point 2).
13. Status values: the header and the Rust constants agree (FFI-CR-09).
14. A C program built against the new header does new, use, free, double free and a forged token, and checks every status.
15. Miri: `cargo +nightly miri test -p mkpdf-ffi` is a required CI job (FFI-002). It has never run in this repository. Until it runs green, no claim of memory safety is made for this design. Fuzzing the token decoder is optional and cheap.

## Alternatives

### A. Table with indices and generation counter (proposed, owner decision D-11)

For: no memory is read through a caller-supplied value. Stale, forged and wrong-kind tokens are rejected. A busy and a poison check come with it. Fewer `unsafe` blocks (8 down to 2). The token is a plain integer, which every runtime can carry safely and which gives the Go cgo pointer checks nothing to object to.

Against: a global lock, a global `static`, a table with a limit, a slot kept forever when a generation is exhausted, and more code than a raw `Box`. Every ABI function that takes a handle changes. The wrappers must carry a 64-bit unsigned value (Node needs `BigInt`, since `Number` is exact only to 2^53). The caller can still pass a valid token of another live object of the same kind, which the table cannot detect, and the other object is then used. That mistake is a caller bug in every handle scheme.

### B. Magic tag inside the object (ADR-0003's open option)

The object starts with a fixed tag, checked on each call and zeroed on free.

For: smallest change. The pointer type stays.

Against: it reads the memory behind the pointer, which is undefined behaviour for a freed or foreign pointer, and in practice often reads reused memory that holds a live object or other data. After `free` the allocator may reuse the block, so a tag may be valid again. It detects some accidents, and it cannot make a double free safe. It does not meet D-08. Rejected, and it should be removed from ADR-0003's open list.

### C. Registry of live pointers

Keep the raw pointer as the handle, and keep a set (`HashSet<usize>` or a sorted vector) of live pointers under a lock. Check membership before dereferencing.

For: no dereference of an unknown pointer. The pointer type in the ABI is unchanged, so the header change is small. No generation field.

Against: the allocator reuses addresses. A stale pointer to a freed document is valid again when a new document is allocated at the same address, so a use after free silently operates on a different object. That is a safe-looking bug and not an error. It adds a hash set and a lock like option A, and has no generation to prevent it. It also cannot add a kind check without more state. It meets D-08 for double free and foreign pointers and does not meet it for use after free. Not chosen by the owner (D-11 names the generation counter).

### D. Tagged pointer

Put a generation or a kind in the unused bits of a pointer (the high bits, or the low alignment bits).

For: the handle is still pointer-sized, can still be a pointer type in the header, and a cheap check of the tag happens before any dereference. Could be paired with a generation stored in the object.

Against: a tag only helps if the library can validate it without reading the object. For that it needs a table or a registry as in options A and C, and then the pointer bits are decoration. Without a table, the generation lives in the object, which is option B. Also: high-bit tagging is not portable. 32-bit targets have no spare bits, ARM pointer authentication and CHERI use those bits, and some platforms use them for address-space layout. Low alignment bits give only 3 or 4 bits. A tagged value is no longer a valid pointer, so a pointer-typed field breaks pointer-aware tools (sanitizers, the Go cgo checks) and invites code that dereferences it. Rejected.

### E. Status quo: contract only

Keep raw pointers and document that misuse is undefined behaviour. Rejected by D-08.

### Comparison

| | Reads memory through caller value | Detects double free | Detects use after free | Detects foreign value | Detects wrong kind | Busy check | `unsafe` blocks | ABI change |
|---|---|---|---|---|---|---|---|---|
| A table + generation | no | yes | yes | yes | yes (kind field) | yes | 2 | yes, breaking |
| B magic tag | yes (UB) | not reliably | not reliably | not reliably | no | no | 8 | no |
| C live-pointer registry | no | yes | no (address reuse) | yes | no | possible | about 6 | small |
| D tagged pointer | depends: yes without a table | as B or A | as B or A | as B or A | partial | no | 8 or fewer | yes |
| E status quo | yes | no | no | no | no | no | 8 | no |

The `unsafe` counts are estimates from the design and are not measured.

## Consequences

- Memory safety for handles no longer depends on the caller. The caller can still break it with the other raw pointers (an out pointer, a string pointer, the buffer data pointer).
- New code in the FFI crate: a table module with its own tests, two helpers, and a changed signature on each export. The size of this work is the FFI-003 backlog item.
- The header, `docs/api/c-abi.md`, `docs/api/rust-api.md` where it mentions handles, the threat model (T-03 changes from "mitigated by contract" to "mitigated by validation", still to be re-assessed by the Security Engineer), and the roadmap text for FFI-001 all need updating after acceptance.
- FFI-001 depends on this ADR (FFI-003 comes first, D-08), and the ADR-0012 text should be re-read against it.
- Wrappers: one `uint64` per object. Python `int`, Go `uint64`, .NET `ulong` or a `SafeHandle` holding it, Java `long`, C++ `uint64_t`. Node needs `BigInt` for the full range (or the generation field must be kept below 21 bits, which this ADR does not propose). A wrapper should set its stored token to `0` after a free, so a second free is a cheap no-op (see open point 5). A stale token is now an error status, so wrappers surface it as an exception and never as a crash.
- Threads: Go and .NET need no new rule for pointer pinning of handles, since the handle is an integer. The error-text thread rule (SEC-F-12) is unchanged.
- Performance cost: one short global lock and one `try_lock` per call. Not measured. Needs a measurement before v0.1.0.
- The table is never built for `wasm32` (the FFI crate is not built there, ADR-0009).
- At library unload or process exit, live handles are leaked, as with raw pointers.
- ADR-0003 stays Proposed and is amended as listed above if this ADR is accepted.

## Open points for the owner (NOT DECIDED)

These are not decided. Each has a recommendation from the architect.

| # | Point | Options | Recommendation |
|---|---|---|---|
| 1 | Token type and name in the header | `uint64_t` integer; or a pointer-typed token that is never dereferenced | `uint64_t`. A pointer type that is not a pointer misleads tools and users (alternative D). Name: keep `MkpdfDocument` as the typedef name, or use `MkpdfDocumentHandle`. The architect has no strong view on the name |
| 2 | Value written to `*out_index` on failure in `add_page` (and the other out values) | leave unwritten; write `usize::MAX`; write `0` | `usize::MAX`, documented as "no page", because `0` is a valid page index and an unwritten value is the garbage problem of FFI-CR-02. Out tokens and `out_count` get `0`. `MAX_PAGES` is 100 000, so `usize::MAX` can never be a real index |
| 3 | Concurrent use of one handle: busy status or blocking | return `BUSY` at once; wait for the lock | Return `BUSY`, as in section 3 |
| 4 | ABI version rule while the major is 0 (ARCH-F-03) | any non-additive change raises the major (this gives 1.0 now, which contradicts "frozen at 1.0.0"); or raises the minor, with an exact major-and-minor match in bindings | Minor, exact match. This change makes the ABI 0.2. Needs an amendment of ADR-0004 and `rules/interoperability.md` item 7 |
| 5 | New status codes and their values, and what `free(0)` does | codes 5 and 6 as in section 6; `free(0)` as a no-op returning `OK`, or returning `INVALID_HANDLE` | Codes 5 (`INVALID_HANDLE`) and 6 (`BUSY`). `free(0)` is a no-op returning `OK`, so a wrapper that zeroes its token after a free can call free again without an error. Any other repeat returns `INVALID_HANDLE` |
| 6 | `MAX_LIVE_HANDLES` per kind | a number up to 2^24 | Not chosen. The Security Engineer and the owner pick a value and each must have a boundary test. Memory is about 16 bytes per slot, so 2^20 slots is about 16 MiB at most |
| 7 | Whether `free` on a handle that another call is using may succeed | free removes it and the running call finishes (section 5); or free returns `BUSY` | Free succeeds. A caller must always be able to release a handle |

## Not examined

- No code was written or run for this ADR. The design and the counts are from reading `lib.rs` at `a57d899`.
- The behaviour of `std::sync::Mutex` poisoning and `try_lock`, and the Miri support for threads, are from knowledge of the standard library and have not been run here. Miri is not installed in this environment.
- The performance cost is not measured.
- Security Engineer review of this ADR has not happened.
