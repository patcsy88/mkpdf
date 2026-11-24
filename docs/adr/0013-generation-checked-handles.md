# ADR-0013: Generation-checked handles

Status: Proposed. Not accepted. Only the project owner accepts it. D-19 says it stays Proposed until the Security Engineer has reviewed this amended text, and then the owner accepts it. FFI-003a is this decision. FFI-003b (the code) starts after acceptance (D-22).

Implements owner decisions D-08, D-11, D-13 to D-18 and D-20. Amends ADR-0003 (see "What this amends"). Related: ADR-0004, ADR-0011, ADR-0012, ARCH-F-01, ARCH-F-03, FFI-CR-03 to FFI-CR-05, SEC-F-04, SEC-ADR-01 to SEC-ADR-05, SEC-ADR-14, SEC-ADR-15, threat model T-03.

What is decided and what is not:

- Decided by the owner and recorded here: D-08, D-11, D-13 to D-18, D-20 (see "Decisions taken by the owner").
- **NOT DECIDED**, and only these: (1) acceptance of this ADR, (2) the numeric value of `MAX_OUTPUT_BYTES` and of the live-buffer byte budget, which D-20 requires but does not give a number for (set in ADR-0012, listed again under "Still open").
- Nothing in this ADR has been implemented or run. Every statement about code is a design statement.

## Changes since first draft

The first draft (commit ccc73d5) had seven open points. This revision:

| Change | Reason |
|---|---|
| Open points 1 to 5 and 7 are now recorded as decided (D-13 to D-18) | Owner decisions in `docs/requirements.md` |
| `MAX_LIVE_HANDLES = 65 536` per kind, a hard cap of `1 << 24` slots, and a byte budget for live buffers | D-20, SEC-ADR-01 rule 1, SEC-ADR-05 |
| Four table rules added (checked index, `get` not `[]`, checked generation bump, retired slot stays empty) | SEC-ADR-01 |
| `new` allocates before it takes the table lock. Every export creates and drops its `Arc` clone and lock guard inside `catch_unwind`. `Mutex::clear_poison` is forbidden | SEC-ADR-03 |
| Retirement wording corrected ("minutes", not "impractical") | SEC-ADR-02 |
| Residual hazards listed: panic hook, signal handler, `fork` | SEC-ADR-04 |
| Table growth uses `try_reserve`. Aggregate buffer byte budget added | SEC-ADR-05 |
| `BUSY` applies to every document call, including read-only ones. Wrappers must not retry silently | SEC-ADR-15 |
| Buffers: stored as `Arc<[u8]>`, one `view` accessor, pointer lifetime rule that resolves the conflict with D-18 | SEC-ADR-11 |
| Wrong-object hazard and wrapper binding rules (Node `BigInt`, test `!= 0`) | SEC-ADR-14 |
| The Miri statement is re-based on CI run 45103722429 | RISK-F-P2-02 |
| Claims reworded so that nothing is called "tested" or "closed" before a test exists | SEC-ADR-13 |
| Test list extended (3 new tests, 3 changed) | SEC-ADR-01, 03, 05, 07 |

### Findings not accepted, or accepted only in part

| Finding | Disposition | Reason |
|---|---|---|
| SEC-ADR-02, optional FIFO free list | Not adopted. LIFO stays | Retirement already removes ABA reuse completely, so FIFO gives no safety gain. It adds a `VecDeque` for a cost argument only. It can be changed later without an ABI change. If the owner wants it, it is a one-line change in section 2 |
| SEC-ADR-04, "document in `c-abi.md`" | Recorded here, not written into `c-abi.md` by this change | The `c-abi.md` text is owned by the interop agent and is being edited in the working tree. This ADR states the rule. The interop agent writes the sentence with FFI-003b |
| SEC-ADR-14, binding rules and a high-generation test constructor | Recorded here as rules. The constructor is a test requirement | The wrapper text belongs to the interop agent and to the first SDK |

## Context

- D-08 (owner): a double free, a use after free and a foreign pointer must return an error and never cause undefined behaviour. The ABI is 0.x, so the change is allowed now.
- D-11 (owner): the mechanism is table indices with a generation counter, so a forged or stale handle is never dereferenced.
- Today (`crates/mkpdf-ffi/src/lib.rs` at `a57d899`) a handle is a `Box::into_raw` pointer to `MkpdfDocument`. A pointer cannot be checked without reading through it, and reading freed or foreign memory is itself undefined behaviour (SEC-F-04, FFI-CR-04). So no check can be added to the current design. The handle has to stop being something the library dereferences.
- The one-thread-at-a-time rule for a handle is only a written contract today (`docs/api/c-abi.md`, ADR-0003). Nothing detects a violation.
- Eight non-test `unsafe` blocks existed at `a57d899` (FFI-CR-15). Five of them exist only because a raw handle is turned into a reference.
- ADR-0012 proposes a second handle type, the output buffer. ADR-0011 proposes a poison rule after a caught panic. This ADR has to serve both.

## Decisions taken by the owner

These are recorded as decided. The architect's earlier recommendations matched all of them.

| ID | Point | Decision |
|---|---|---|
| D-13 | Token type | `uint64_t` token, not a fake pointer. Token 0 is never valid |
| D-14 | Out parameters on failure | A neutral value is written first: `usize::MAX` for a page index, 0 for tokens and counts |
| D-15 | Concurrent use of one handle | Return `BUSY`, do not block |
| D-16 | ABI 0.x versioning | Exact major and minor match while 0.x. The ABI moves to 0.2 with this change. ADR-0004 and `rules/interoperability.md` item 7 need amending (not done yet, owner approval needed for the rules file) |
| D-17 | Status codes | 5 = `INVALID_HANDLE`, 6 = `BUSY`. `free(0)` is a no-op returning `OK` |
| D-18 | `free` on a handle in use | `free` succeeds and the running call finishes |
| D-20 | Limits | `MAX_LIVE_HANDLES` = 65 536 per handle kind, a table cap of 2^24 slots, and a total byte budget for live buffers. The table size estimate is not measured |

## What this amends in ADR-0003

ADR-0003 is Proposed and is not edited by this change. If the owner accepts this ADR, ADR-0003 should be marked Superseded in part, or amended in the same commit, as follows:

| ADR-0003 text | Change |
|---|---|
| "Objects are opaque pointers created and freed by the library" | Objects are referred to by opaque 64-bit tokens. The library still owns them, and the caller releases each with the matching `*_free` |
| "`free(NULL)` is a no-op" | `free(0)` is the equivalent (D-17) |
| "A handle is used by one thread at a time" | Unchanged as a rule for the caller. It is now also enforced: a second concurrent call gets `BUSY` instead of a data race |
| Consequences: "Double free and use after free are undefined behaviour in C" | No longer true for handles. They return an error status. Memory safety no longer rests on the caller contract for handle validity. It still rests on it for the other pointers (see "Unsafe blocks") |
| Consequences: "Debug-time handle validation (for example a magic tag) is an open option for GATE-1" | Closed. A magic tag inside the object is rejected (alternative B) |
| Unchanged | Status codes as the return value, per-thread error text, UTF-8 pointer-plus-length strings, panics caught at the boundary (ADR-0011), library-owned objects |

## Decision (proposed)

### 1. Token type and layout (D-13)

A handle is a `uint64_t` value. It is never a pointer and is never dereferenced.

```
bit 63            32 31      24 23                 0
    | generation    |  kind    |       index       |
```

- `index` (24 bits): the slot number in the table for that kind. At most `1 << 24` slots exist per kind (D-20).
- `kind` (8 bits): which table. 1 = document, 2 = buffer (ADR-0012). Kind 0 is invalid. The kind makes a document token passed to a buffer function fail on the kind, not succeed because the numbers happen to line up.
- `generation` (32 bits): starts at 1 for every slot and rises each time the slot is freed. A slot never has generation 0, so the token `0` is never valid and is the "no handle" value.
- The C type is `typedef uint64_t MkpdfDocument;`. Functions take `MkpdfDocument doc` by value, and `new` takes `MkpdfDocument *out`. Whether the typedef keeps its name is a header naming point, not a design point.
- Decoding is shifts and masks on a `u64`. It cannot panic. The token constructor takes a `u32` index that has been checked to be below `1 << 24`. It must not use `as` truncation (SEC-ADR-01 rule 1).

### 2. Table structure and locking

One table per kind, in the FFI crate only. `mkpdf-core` is untouched (no global state in core, `rules/architecture.md` item 2).

```rust
struct Slot<T> { generation: u32, value: Option<Arc<T>> }
struct Table<T> { slots: Vec<Slot<T>>, free: Vec<u32>, live: usize }
static DOCS: Mutex<Table<Mutex<Document>>> = Mutex::new(Table::new());
const MAX_LIVE_HANDLES: usize = 1 << 16;   // D-20, per kind
const MAX_SLOTS: usize = 1 << 24;          // D-20, hard cap, equals the index space
const _: () = assert!(MAX_LIVE_HANDLES < MAX_SLOTS);
```

Rules that every table operation follows (SEC-ADR-01):

1. **Slot cap.** `live <= MAX_LIVE_HANDLES` bounds live objects. It does not bound `slots.len()`, because retired slots stay in `slots` for good. So `new` also returns `MKPDF_ERR_LIMIT_EXCEEDED` when the free list is empty and `slots.len() == MAX_SLOTS`. No truncated index is ever issued.
2. **Lookup uses `slots.get(index)`, never `slots[index]`.** The workspace sets `indexing_slicing` to warn, not deny. A forged token must never reach a panic.
3. **The generation bump is checked.** `overflow-checks = true` in release, so a plain `+= 1` at `u32::MAX` would panic in the middle of `free`. The order in `free` is: check the token, `take()` the value, then either retire the slot (generation is `u32::MAX`, not pushed on the free list) or bump with `checked_add` and push the index, then release the table lock, then drop the `Arc`.
4. **A retired slot keeps `value = None` and generation `u32::MAX` for good.** Lookup checks `kind`, then `index`, then generation, then `value`, and treats `None` as `INVALID_HANDLE`. A forged token with generation `u32::MAX` matches the generation and fails on the value.

Operations:

- `new`: allocate `Arc::new(Mutex::new(Document::new()))` **before** taking the table lock. Then lock the table, check the limits, take an index from `free` or push a slot, store the `Arc`, set `live += 1`, build the token, release the lock. If a limit check fails, release the lock first and then drop the `Arc`. The table grows with `try_reserve`, and a failed reservation returns `LIMIT_EXCEEDED`, not an abort (SEC-ADR-05). On failure the out token is `0` (D-14).
- Lookup (every function except `new`): lock the table, decode the token, apply rules 2 and 4, clone the `Arc`, release the table lock. Any failed check returns `MKPDF_ERR_INVALID_HANDLE`. No memory is read through the token.
- The table lock covers only index arithmetic, `Vec` growth and one `Arc` clone. No document code, no document allocation and no document drop runs under it. So one lock order exists: the table lock is never held while a document lock is taken, and the two are never held together.
- A panic while the table lock is held is not expected. If the table mutex is poisoned anyway, recover with `PoisonError::into_inner`. This is sound only because every table step leaves the invariants intact at every point where it could panic. Rules 1 to 4 are what make that true, so they are a precondition of this recovery.
- The table is a global `static`, so every call takes one short global lock. Sharding or a read-write lock is a later change that needs no ABI change. The cost is not measured.

### 3. Concurrency: busy status, not blocking (D-15)

After lookup the call does `try_lock` on the document's own mutex.

- If the lock is free, the call runs. If another call holds it, the call returns `MKPDF_ERR_BUSY` at once.
- This applies to **every** document call, including `page_count` and `save`. Two concurrent reads of one document return `BUSY` for one of them. That matches the one-thread-at-a-time contract. Buffers take no mutex and never return `BUSY` (SEC-ADR-15).
- The reason for `BUSY`: a caller that breaks the contract has a bug. Blocking would hide it and turn it into latency, and could deadlock if a wrapper holds its own lock across the call.
- **Wrappers must surface `BUSY` as an error. They must not retry in a loop.** A silent spin turns a caller bug into a CPU-bound hang.
- Distinct handles never contend on a document lock. They share only the short table lock.
- A handle may be used from different threads one after another. No thread affinity exists.
- `free` while a call runs: `free` takes the slot out of the table first, so later lookups fail with `INVALID_HANDLE`. A call that already cloned the `Arc` finishes on a valid object, and the document is dropped when the last clone is dropped (D-18).
- `try_lock` returns `WouldBlock` (map to `BUSY`) or `Poisoned` (map to `MKPDF_ERR_INTERNAL`, and drop the guard that came with the error). These are from knowledge of `std`, not run here.

### 4. Guard scope and poisoning (link to ADR-0011)

- **Scope.** Every export creates its `Arc` clone and its `MutexGuard` **inside** the `catch_unwind` closure, and both are dropped inside it. With D-18 the last clone can be dropped by any call, not only by `free`, so a `Drop` that panics must run inside the guard. Otherwise it runs outside any `catch_unwind` and aborts the host (SEC-ADR-03, ADR-0011 item 7). Today `Document` has no panicking drop. That is a property to keep, not a fact to rely on once the writer adds types.
- **Poison.** If the closure panics, unwinding drops the guard and the standard `Mutex` marks itself poisoned. No extra flag is needed. This is the poison flag of ADR-0011 item 4, obtained from the standard library.
- Every later call on that handle sees the poisoned lock and returns `MKPDF_ERR_INTERNAL`. `free` still works on a poisoned handle.
- **`Mutex::clear_poison` is forbidden** in the FFI crate. One call would silently remove the poison rule. Enforcement is proposed as a clippy `disallowed-methods` entry (a config file, so it belongs to rust-core or devops, not to this change).
- This depends on `panic = "unwind"` (ADR-0011). Under `abort` there is nothing to poison, which is one more reason for the `compile_error!` there.
- A panic in a function that holds no document lock (for example `new`) leaves nothing to poison. The table is updated only after the fallible work is done.
- It does not protect against a panic after the guard is released. Keep all document mutation inside the locked section.

### 5. Free semantics (D-17, D-18)

`mkpdf_document_free(MkpdfDocument doc)` returns `int32_t`.

- Valid token: remove the value from the slot, bump or retire the generation (section 2 rule 3), push the index on the free list if not retired, release the table lock, then drop the `Arc`. Returns `MKPDF_OK`.
- **Retirement.** A slot whose generation is `u32::MAX` is retired and never reused. A caller that loops `new` and `free` reuses one slot each time (the free list is LIFO), so about 2^32 cycles retire a slot. At a few million cycles per second that is minutes to tens of minutes, not years (SEC-ADR-02). This is safe: a retired slot costs about 16 bytes (estimated, not measured), and the slot cap in section 2 is reached only after about 16 million retirements. Retirement is a required test, not an optional one.
- Stale, already freed, foreign or wrong-kind token: `MKPDF_ERR_INVALID_HANDLE`. No memory is touched.
- Token `0`: a no-op returning `MKPDF_OK` (D-17). A wrapper that zeroes its stored token after a free can free again without an error. Any other repeat returns `INVALID_HANDLE`.
- Free works on a poisoned or busy document. If the document is busy on another thread, free still removes it from the table and the running call completes on its own clone (D-18).
- Two threads freeing the same token at once: one gets `OK`, the other `INVALID_HANDLE`.
- `free` takes only a token, so it needs no `unsafe` and can be a safe `extern "C" fn`.

### 6. Status codes (D-17)

Additive. Values never change once released.

| Value | Name | Meaning |
|---|---|---|
| 5 | `MKPDF_ERR_INVALID_HANDLE` | The token is zero (where zero is not allowed), stale, foreign, or of the wrong kind |
| 6 | `MKPDF_ERR_BUSY` | The handle is in use by another call. The caller broke the one-thread-at-a-time rule. Do not retry in a loop |

`MKPDF_ERR_NULL_POINTER` stays for out parameters and strings. It no longer applies to a handle, because a handle is not a pointer. `MKPDF_ERR_INTERNAL` (100) also covers a poisoned handle, and the error text distinguishes the cases. The header, `docs/api/c-abi.md`, the Rust constants and the SDK status tables all gain the two rows. A test compares them (FFI-CR-09).

### 7. Effect on ADR-0012 (output buffers)

- `MkpdfBuffer` is a token of kind 2 in a second table, `Table<[u8]>` holding `Arc<[u8]>`. A buffer is immutable, so it needs no per-object mutex and never returns `BUSY`. Many threads may read it. Bytes are held as a boxed slice, so the buffer cannot grow, the pointer never moves, and no spare capacity is kept (SEC-ADR-11).
- ADR-0012 defines the accessors. In short: one `mkpdf_buffer_view(buf, &data, &len)` call returns both values from one lookup, and `mkpdf_buffer_free` returns a status.
- **Pointer lifetime and D-18.** For documents, D-18 is safe because the running call owns an `Arc` clone. For buffers the returned pointer outlives the call, so a free on another thread could drop the bytes before the caller reads them. The rule for buffers is: the pointer from `view` is valid until `mkpdf_buffer_free` **starts** on any thread, and the caller orders its own free after its last read. `free` still succeeds (D-18). This is a caller contract that no table can enforce, because the caller holds a raw pointer. It is the one place where memory safety still rests on the caller after this ADR, and it is the same class as T-02, not T-03.
- A generic `Table<T>` is justified because two uses exist (documents and buffers, `rules/architecture.md`: no abstraction without a second use). If ADR-0012 is not accepted, the table stays a plain document table.
- The out-parameter rule applies to buffers: `mkpdf_document_save` writes `0` to the out token first (D-14).
- Buffers are counted against the byte budget (D-20) under the buffer table lock when registered, and released when freed. ADR-0012 gives the rule and the value.

### 8. ABI version impact (D-16)

- Every function that takes or returns a handle changes: `new`, `free`, `set_title`, `add_page`, `page_count` and everything added later. The header changes. This is an incompatible ABI change.
- Decided: while the major is 0, any change that is not a pure addition raises the minor, and bindings accept only an exact major-and-minor match. This change moves the ABI from 0.1 to **0.2**. It avoids the contradiction in ARCH-F-03 (a breaking change would otherwise make the ABI 1.0, which is "frozen").
- Still to do: ADR-0004 and `rules/interoperability.md` item 7 need the matching amendment. The rules file needs the owner's approval, so this ADR does not edit it.
- Nothing has been released, so no installed user is affected. The cost lands on wrappers that do not exist yet.

### 9. What happens to the unsafe blocks

Today: 8 non-test `unsafe` blocks (FFI-CR-15, at `a57d899`). After this ADR, with the helpers proposed in FFI-CR-03:

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

Result: **2** non-test `unsafe` blocks (`write_out` and `from_raw_parts`), down from 8. Without the `write_out` helper it is 4. ADR-0012 adds one more for `mkpdf_buffer_view` (two out writes through `write_out`, so no new block). These counts come from a design, not from code. The remaining blocks are exactly the ones that handle caller-supplied raw pointers, which no table can check. What the change removes is the class of bug where a stale handle becomes a dangling reference.

- Functions that take only tokens may no longer need to be `unsafe extern "C" fn`. Whether a function that takes a raw out pointer stays `unsafe` is a Rust-side choice for the FFI crate.
- The `#![allow(unsafe_code)]` can move from the crate to the two helpers and the exports (check first that `#[unsafe(no_mangle)]` does not need the allow, FFI-CR-13).

### 10. Residual hazards (SEC-ADR-04)

No table design removes these. They are host bugs or edge cases, and none is undefined behaviour inside this library. `docs/api/c-abi.md` must say each one in a sentence (interop agent, with FFI-003b).

- Do not call the library from a signal handler. It is not async-signal-safe.
- Do not call the library from a Rust panic hook. The hook runs before unwinding and may run with the table lock held on the same thread, so `Mutex::lock` can deadlock or panic.
- After `fork()` in a multi-threaded host, call only `exec`. If another thread held the table lock at the moment of the fork, every call in the child would block for ever.
- The thread-local error state is a `RefCell` used with `try_borrow_mut` and `try_with`. It is not a lock, it is per thread, and it is touched after the closure returns with no table or document lock held. It cannot deadlock with the table.
- No callbacks exist (ADR-0012 rejects them), so no re-entry arises through the ABI.

### 11. Wrong-object use and wrapper rules (SEC-ADR-14)

- The table cannot detect a valid token of **another live object of the same kind**. If a wrapper corrupts a token into another valid one, a call silently changes the wrong document. In a server that is a confidentiality and integrity problem across requests, so wrappers must treat it as a security rule.
- Node: a token is a `BigInt` only. A `Number` loses the low bits once the generation reaches 2^21 (about two million frees of one slot), and the rounded value can name a different slot with the same generation.
- Every wrapper tests a token with `!= 0`, never with `> 0`. In Java a `long` token is negative when bit 63 is set (generation at or above 2^31).
- Binding tests must start a table at a high generation (a test-only constructor, section 12 test 6) to catch both bugs.
- These rules go into `docs/api/c-abi.md` (interop agent), before the first SDK.

### 12. Required tests

All in the FFI crate and run under Miri. **None exists yet.** No test below is passing today.

1. Lifecycle: new, use, free, with the token non-zero and kind and generation as designed.
2. Double free: second `free` returns `INVALID_HANDLE`. Document and buffer.
3. Use after free: every function that takes a handle returns `INVALID_HANDLE` on a freed token.
4. Slot reuse: after free and new, the new token differs from the old one, the old token is still rejected and the new one works.
5. Forged tokens: zero, `u64::MAX`, an index beyond the table, an index equal to `slots.len()` exactly, a valid index with the wrong generation (plus and minus one), a token naming a retired slot with generation `u32::MAX`, kind 0, an unknown kind, and the token of the other kind. A property test over random `u64` values never panics and never returns `OK` for a token that was not issued.
6. Generation retirement: a table test with the generation near `u32::MAX` shows the slot is retired and not reused, and `free` does not panic at the boundary. Needs a test-only constructor that starts a slot at a chosen generation. The constructor is gated by `--cfg mkpdf_test_hooks` (ADR-0011 item 3), not by a Cargo feature.
7. Limit: with a small `MAX_LIVE_HANDLES` in a test table, the one-past `new` returns `LIMIT_EXCEEDED`, writes `0` to the out token, and a `free` makes room again. A `const` assertion checks that the real constant is `1 << 16` and below `1 << 24`.
8. Slot cap: fill `slots` to the cap with retired slots in a test table. `new` fails with `LIMIT_EXCEEDED` and no truncated index is issued.
9. Busy: a test holds the document lock and calls an export. It must return `BUSY` and not block. After release the call works. Also check a read-only call (`page_count`) and a second `save`.
10. Concurrency: several threads each use their own handle, and several threads race to free one token (exactly one `OK`). Run under Miri with threads.
11. Free while a call is running, and the call holds the last clone: the running call completes, the document is dropped **inside** `guard`, and no use after free occurs (Miri). A test document type with a panicking `Drop` shows the abort does not happen outside the guard.
12. Poison: a test-only panic inside a locked call (gated by `--cfg mkpdf_test_hooks`) makes later calls return `MKPDF_ERR_INTERNAL`, and `free` still returns `OK`. It runs against the release `cdylib` through a real export (ADR-0011 item 3).
13. Out parameters: each function writes the documented neutral value on every failure path (D-14).
14. Status values: the header and the Rust constants agree (FFI-CR-09).
15. A C program built against the new header does new, use, free, double free and a forged token, and checks every status.
16. Miri: `cargo miri test -p mkpdf-ffi` is a required CI job (FFI-002). See "Evidence from CI" for what has run so far.
17. Byte budget (ADR-0012): see ADR-0012 tests.

Fuzzing the token decoder is optional and cheap.

### 13. Evidence from CI

CI run 45103722429 (push, commit ccc73d5, 2026-11-23) is green on all 10 jobs, read from the GitHub API and the job log. The `miri-ffi` job installed `rustc 1.102.0-nightly (137d1d494 2026-11-22)` and ran `cargo miri test -p mkpdf-ffi --locked`: 12 tests passed. The `msrv` job installed `rustc 1.85.0` and passed. So Miri and MSRV both ran, and the earlier statement "Miri has never run" is no longer true. What Miri has checked is the **current** FFI crate and its 12 tests: no table, no threads and no ADR-0013 test. So no claim of memory safety for this design is made. The statement that Miri supports the threaded tests of this design is still unverified until those tests exist.

## Alternatives

### A. Table with indices and generation counter (proposed, owner decision D-11)

For: no memory is read through a caller-supplied value. Stale, forged and wrong-kind tokens are rejected. A busy and a poison check come with it. Fewer `unsafe` blocks (8 down to 2, by design). The token is a plain integer, which every runtime can carry safely and which gives the Go cgo pointer checks nothing to object to.

Against: a global lock, a global `static`, a table with limits, a slot kept forever when a generation is exhausted, and more code than a raw `Box`. Every ABI function that takes a handle changes. Wrappers must carry a 64-bit unsigned value (Node needs `BigInt`). The caller can still pass a valid token of another live object of the same kind, which the table cannot detect (section 11).

### B. Magic tag inside the object (ADR-0003's open option)

The object starts with a fixed tag, checked on each call and zeroed on free.

For: smallest change. The pointer type stays.

Against: it reads the memory behind the pointer, which is undefined behaviour for a freed or foreign pointer, and in practice often reads reused memory that holds a live object or other data. After `free` the allocator may reuse the block, so a tag may be valid again. It detects some accidents and cannot make a double free safe. It does not meet D-08. Rejected, and it should be removed from ADR-0003's open list.

### C. Registry of live pointers

Keep the raw pointer as the handle, and keep a set of live pointers under a lock. Check membership before dereferencing.

For: no dereference of an unknown pointer. The pointer type in the ABI is unchanged. No generation field.

Against: the allocator reuses addresses. A stale pointer to a freed document is valid again when a new document is allocated at the same address, so a use after free silently operates on a different object. It adds a hash set and a lock like option A, and cannot add a kind check without more state. It meets D-08 for double free and foreign pointers and does not meet it for use after free. Not chosen (D-11 names the generation counter).

### D. Tagged pointer

Put a generation or a kind in the unused bits of a pointer.

For: the handle is still pointer-sized, and a cheap check of the tag happens before any dereference.

Against: a tag only helps if the library can validate it without reading the object. For that it needs a table or a registry as in A and C, and then the pointer bits are decoration. Without a table, the generation lives in the object, which is option B. High-bit tagging is not portable: 32-bit targets have no spare bits, ARM pointer authentication and CHERI use those bits, and low alignment bits give only 3 or 4 bits. A tagged value is no longer a valid pointer, so a pointer-typed field breaks pointer-aware tools and invites code that dereferences it. Rejected. D-13 (`uint64_t`, not a fake pointer) agrees.

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

The `unsafe` counts are estimates from the design and are not measured. The "yes" cells for A describe what the design allows if implemented as written, and are not test results.

## Consequences

- Memory safety for handles will no longer depend on the caller, **once** FFI-003b is implemented and the tests in section 12 pass. The caller can still break it with the other raw pointers (an out pointer, a string pointer, the buffer data pointer).
- New code in the FFI crate: a table module with its own tests, two helpers, and a changed signature on each export. The size of this work is FFI-003b.
- The header, `docs/api/c-abi.md`, `docs/api/rust-api.md` where it mentions handles, the threat model and the roadmap text for FFI-001 all need updating after acceptance. Threat T-03 moves from "mitigated by contract" to "mitigated by validation" only when the tests exist and pass. The Security Engineer re-assesses it.
- New hazards for the threat model, to be added when the owner accepts the ADRs (SEC review): exhaustion of the global table by one leaking component (the table is shared by every user of the library in the process), wrong-object use through a corrupted token, and `fork` or signal-handler calls with the table lock held.
- The 65 536 limit makes a handle leak in a wrapper (a finalizer that never runs, a missing `Dispose`) show as `LIMIT_EXCEEDED` for every new handle in the process, before it becomes an out-of-memory abort. The error text must say "live document handle limit reached". The limit does not bound memory (see ADR-0012 for the byte budget). Per-document memory is bounded by the core limits, and the total is the host's concern.
- FFI-001 depends on this ADR (FFI-003 comes first, D-08). ADR-0012 depends on it for the token shape.
- Wrappers: one `uint64` per object. Python `int`, Go `uint64`, .NET `ulong` or a `SafeHandle` holding it, Java `long`, C++ `uint64_t`, Node `BigInt` only. A wrapper sets its stored token to `0` after a free. A stale token is an error status, so wrappers surface it as an exception and never as a crash.
- Threads: Go and .NET need no new pointer-pinning rule for handles, since the handle is an integer. The error-text thread rule (SEC-F-12) is unchanged.
- Performance cost: one short global lock and one `try_lock` per call. Not measured. Needs a measurement before v0.1.0.
- The table is never built for `wasm32` (the FFI crate is not built there, ADR-0009).
- At library unload or process exit, live handles are leaked, as with raw pointers.
- ADR-0003 stays Proposed and is amended as listed above if this ADR is accepted.

## Still open (NOT DECIDED)

| # | Point | State |
|---|---|---|
| 1 | Acceptance of this ADR | The owner decides, after the Security Engineer has reviewed this amended text (D-19). The Security Engineer's earlier review said "sound enough to accept, with changes". That review was of the first draft |
| 2 | Value of `MAX_OUTPUT_BYTES` and of the live-buffer byte budget | D-20 requires a byte budget and gives no number. Both are set together in ADR-0012. Proposed there, not decided |

Everything else in this document is either decided by the owner (D-08, D-11, D-13 to D-18, D-20) or proposed as part of the whole and not accepted until point 1 is settled.

## Not examined

- No code was written or run for this ADR. The design and the counts are from reading `lib.rs` at `a57d899` and the findings in `docs/reviews/security-adr-0011-0013.md`.
- The behaviour of `std::sync::Mutex` poisoning, `try_lock`, `try_reserve` and Miri's support for threads is from knowledge of the standard library. It was not run here for this design. Local Miri is not installed. CI has run Miri on the current crate only (section 13).
- The table size and the performance cost are not measured.
- The Security Engineer has not reviewed this amended text.
