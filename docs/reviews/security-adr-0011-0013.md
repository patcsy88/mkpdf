# Security review: ADR-0011, ADR-0012, ADR-0013 and FFI-CR-05

- Reviewer: Security Engineer (agent role, `claude-opus-5-5` per my configuration, unverified from inside the session). Single reviewer. This is not an independent second review.
- Date: 2026-11-23
- Subject: `docs/adr/0011-panic-strategy.md`, `docs/adr/0012-output-buffer-ownership.md`, `docs/adr/0013-generation-checked-handles.md` (all Proposed, untracked in the working tree), and the handle-table sketch in `docs/reviews/ffi-code-review-architect.md` FFI-CR-05 (lines 134-153).
- Owner decisions taken into account: D-08, D-11, D-13 to D-18 (`docs/requirements.md:16-26`). D-19 asks me to propose `MAX_LIVE_HANDLES` and to say whether ADR-0013 is sound enough to accept.
- This review accepts nothing. Only the project owner can accept an ADR or a risk, in writing, in the repository. `docs/reviews/gate-1-security.md` is being edited by another session and was not changed by this review.

## Recommendation

| ADR | Recommendation | Short reason |
|---|---|---|
| ADR-0011 panic strategy | **Accept with changes** | `unwind` plus `compile_error!` is the right choice for in-process hosts. The check must be positive (`not(panic = "unwind")`), the test hook must not be a Cargo feature, and the guard scope must cover drops and the panic payload (SEC-ADR-05, 06, 07, 08) |
| ADR-0012 output buffers | **Accept with changes** | Option A is the safest of the three. The sketch still uses pointer types, which contradicts D-13, the output limit must be set below `i32::MAX` with an aggregate budget, and the "T-23 is closed" claim has no test (SEC-ADR-09 to 12) |
| ADR-0013 generation-checked handles | **Sound enough for the owner to accept, with changes** | No token value can cause UB, an out-of-bounds read or an unbounded allocation in the design, provided the table code follows the rules in SEC-ADR-01 to 04. Lock order is single and `try_lock` removes self-deadlock. The changes are additions to the ADR text, not a redesign |

Proposed `MAX_LIVE_HANDLES` (D-19): **65 536 (`1 << 16`) per kind**, the same for documents and buffers, and separately an aggregate byte budget for live buffers. Reasoning is in the section "Proposed MAX_LIVE_HANDLES".

Findings: 0 critical, 0 high, 8 medium, 7 low. No finding is resolved, because no code for any of the three ADRs exists. None of the threats these ADRs address can be marked "Tested" yet (see "Threat model coverage").

## What was examined

- Read in full: `AGENTS.md`, `rules/security.md`, `rules/interoperability.md`, `docs/requirements.md`, `docs/security/threat-model.md`, `docs/reviews/gate-1-security.md`, `docs/reviews/ffi-code-review-architect.md`, the three ADRs.
- Read for cross-checking claims: `crates/mkpdf-ffi/src/lib.rs` (working tree, 492 lines, includes the in-flight SEC-F-01/02 fix), `crates/mkpdf-ffi/src/lib.rs` at `a57d899` (lines 90-115 and 345-358), `crates/mkpdf-core/src/model.rs` lines 1-130 (limits and `Document`), root `Cargo.toml` and `crates/mkpdf-ffi/Cargo.toml`, `.github/workflows/ci.yml` (grep for `panic` and `miri`), `docs/api/c-abi.md` (grep), `docs/risk/register.md` (grep).
- Checked: the root `Cargo.toml` still has no `panic` key (`Cargo.toml:34-38`); the CI file has no panic-setting check (no match for `panic` in `ci.yml`); the Miri job exists (`ci.yml:74-87`) and has never run.

## What was not examined or run

- No code was written, built or run for this review. I did not reproduce the ADR-0011 measurements (exit codes, `compile_error!` behaviour, artefact sizes). They are marked "verified by the ADR author, not re-run".
- Standard-library and language behaviour I rely on (mutex poisoning on drop during unwinding, `try_lock` results, panic-in-drop during unwinding aborts, abort on unwinding out of `extern "C"` since 1.81, `try_reserve` being stable) is from my knowledge of the Rust documentation. It is not run here. Miri is not installed.
- Sizes (bytes per slot, per empty document) are from reasoning about layouts, not measured with `size_of`.
- No wrapper exists, so every statement about Go, Python, Node.js, .NET, Java and C++ behaviour in ADR-0012 and ADR-0013 is unverified (see "Claims about wrapper and language behaviour").
- Not examined: ADR-0003, ADR-0004, ADR-0007, ADR-0009, ADR-0010 beyond what the three ADRs quote; `docs/reviews/gate-1-architecture.md`; `docs/reviews/risk-gate-1-phase-1.md`; Linux and Windows behaviour; `fork` behaviour on a real system; library unload.
- `cargo deny` and `cargo audit` were not run: these ADRs add no dependency.

## Findings

Severity scale as in `gate-1-security.md`: critical (memory corruption or code execution reachable by hostile input, with no contract violation needed), high (crash or UB reachable through plausible misuse, or a blocking control missing), medium (defence in depth missing, or a gate control unverified), low (hardening, documentation or process). A finding against a design is rated by what the design allows if implemented as written.

| ID | Severity | ADR | Title | Disposition (proposed) |
|---|---|---|---|---|
| SEC-ADR-01 | medium | 0013 | Token decoding and table arithmetic must be stated as panic-free and checked | Amend ADR-0013 before acceptance |
| SEC-ADR-02 | low | 0013 | Generation retirement is reachable in days, not "impractical" | Amend wording; keep test 6 |
| SEC-ADR-03 | medium | 0013 | Where allocations and drops run is contradictory and partly outside `guard` | Amend ADR-0013 before acceptance |
| SEC-ADR-04 | low | 0013 | Lock order is sound; residual re-entry (panic hook, signal handler) and `fork` hazards | Document in ADR-0013 and `c-abi.md` |
| SEC-ADR-05 | medium | 0013, 0012 | `MAX_LIVE_HANDLES` bounds the table, not memory | Add an aggregate byte budget for live buffers; owner sets values |
| SEC-ADR-06 | medium | 0011 | `compile_error!` condition is negative, and the panic test must run on the release artefact, not `cargo test` | Amend ADR-0011 before acceptance |
| SEC-ADR-07 | low | 0011 | A test-only panic hook behind a Cargo feature can leak into a shipped build | Use a `--cfg` flag and a symbol check |
| SEC-ADR-08 | medium | 0011 | Abort paths and side effects that `guard` does not cover are not all listed | Amend ADR-0011 before acceptance |
| SEC-ADR-09 | low | 0011 | The interim rule "validate first, mutate last" is not enforced if the writer lands before FFI-003 | Sequencing rule for the Orchestrator |
| SEC-ADR-10 | medium | 0012 | The ABI sketch still uses pointer types, which contradicts D-13 and makes rule 1 impossible | Amend ADR-0012 before acceptance |
| SEC-ADR-11 | medium | 0012, 0013 | Buffer data pointer lifetime under D-18, and a race between separate `data` and `len` calls | Amend ADR-0012 before acceptance |
| SEC-ADR-12 | medium | 0012 | `MAX_OUTPUT_BYTES` is unset, the wrapper integer limits are not tied to it, and OOM aborts | Set the value in ADR-0012; use `try_reserve` |
| SEC-ADR-13 | low | all | Claims of closure and of safety that no test or run supports yet | Reword, see "Claims" table |
| SEC-ADR-14 | low | 0013 | A valid token of another live object is accepted; Node `Number` precision loss can produce one | Node binding rule and test |
| SEC-ADR-15 | low | 0012, 0013 | `BUSY` on read-only calls and on concurrent `save` is not stated; retry loops | Document; wrappers must not retry silently |

## Findings in detail: ADR-0013 (handles)

### SEC-ADR-01 (medium): token decoding and table arithmetic must be panic-free and checked

- Evidence: `0013:36-43` (layout), `0013:57-58` (new, lookup), `0013:87-88` (free, retirement). Lints: `Cargo.toml:27` sets `indexing_slicing = "warn"`, not deny. `Cargo.toml:37` sets `overflow-checks = true`, so `generation += 1` at `u32::MAX` panics in release builds.
- Analysis of forged tokens against the design as written. Every decoded field is range-checked before use, and nothing is read through the value. So no `u64` can cause UB or an out-of-bounds read, **provided** the implementation follows four rules that the ADR does not yet state:
  1. **Index aliasing on push.** `live <= MAX_LIVE_HANDLES` bounds live objects, not `slots.len()`. Retired slots stay in `slots` forever, so `slots.len()` can grow past `MAX_LIVE_HANDLES`. If it ever reached `1 << 24`, a token built from the new index would be truncated to 24 bits and would name **another slot**. That would make a fresh token refer to a different live object. The ADR must require a second cap: `new` returns `LIMIT_EXCEEDED` when the free list is empty and `slots.len() == 1 << 24`. The token constructor must take a checked `u32` index below `1 << 24`, and must not use `as` truncation.
  2. **Lookup uses `slots.get(index)`, never `slots[index]`.** Indexing is only a warning in this workspace. A panic under the table lock is recoverable (into_inner), but it must not be the normal path for a forged token.
  3. **Generation bump uses an explicit comparison or `checked_add`.** With `overflow-checks = true`, a naive `+= 1` at `u32::MAX` panics under the table lock, in the middle of `free`, after the value was taken. Order of `free` must be: check the token, `take()` the value, then either retire the slot (generation at `u32::MAX`, not pushed on the free list) or bump it and push the index. Then release the lock, then drop the `Arc`.
  4. **A retired slot keeps `value = None` and the generation `u32::MAX` for good.** A forged token with generation `u32::MAX` then matches the generation and fails on `value.is_none()`. The lookup must check `value` after the generation and treat `None` as `INVALID_HANDLE`. This order is already in the ADR (`0013:58`). Keep it and test it.
- Other checks of the token, all sound as designed: token `0` has generation 0, which no slot ever has. Kind 0 and unknown kinds are rejected before the table is chosen. Bits 24-31 select the table, so a document token in a buffer function fails on the kind. Decoding is shifts and masks on a `u64` and cannot panic.
- Unbounded allocation from a token: none. Lookup and free allocate nothing. `new` is the only path that grows the table, and rule 1 above bounds it.
- Disposition: add rules 1 to 4 to ADR-0013 section 2 and section 5 before acceptance. Add to test 5: a token whose index is `slots.len()` exactly, a token naming a retired slot with generation `u32::MAX`, and a table test that fills `slots` to the cap and checks that `new` fails and no truncated index is issued. Owner: architect (ADR text), rust-core (code), qa (tests). Date: before acceptance of ADR-0013.

### SEC-ADR-02 (low): generation retirement is reachable in minutes

- Evidence: `0013:88` says retirement "takes about four billion frees of one slot, so it is a safety rule, not a practical cost".
- Analysis: the free list is a `Vec` used as a stack (LIFO), so a caller that loops `new` and `free` reuses **one** slot every time. At a few million cycles per second, 2^32 cycles take minutes to tens of minutes, not years. That is not a vulnerability. Retirement removes ABA reuse completely, because a generation is never repeated in a slot. Each retirement costs one slot (about 16 bytes, estimated, not measured) per 2^32 frees, so the cap in SEC-ADR-01 rule 1 is reached only after about 16 million retirements, which is years of a tight loop. The design holds. The text overstates how rare the case is, and the case is reachable by a plain client loop, so test 6 is required, not optional.
- Disposition: reword `0013:88`. Keep test 6. Optionally use a FIFO free list (`VecDeque`) to spread generations over slots. That is not required. Owner: architect. Date: with the ADR amendment.

### SEC-ADR-03 (medium): where allocation and drop run is contradictory, and partly outside `guard`

- Evidence: `0013:57` (`new`: "lock the table, take an index ... store `Arc::new(Mutex::new(Document::new()))`") and `0013:59` ("no allocation of a document and no drop of a document runs under it") contradict each other. `0013:72` and `0013:91` (D-18): the running call that holds the last `Arc` clone drops the document. Current `guard` runs `set_last_error` and `failure.message()` outside `catch_unwind` (`crates/mkpdf-ffi/src/lib.rs:103-118`).
- Analysis:
  - Allocate the `Arc<Mutex<Document>>` before taking the table lock, and drop it (on the limit path) after releasing the lock. Then the table lock covers only index arithmetic, which is what section 2 says.
  - With D-18, the last `Arc` can be dropped by any call, not only by `free`. So every export must create its `Arc` clone and its `MutexGuard` **inside** the `catch_unwind` closure, and both must be dropped inside it. Otherwise a `Drop` that panics runs outside the guard and aborts the host. Today `Document`'s drop cannot panic (a `String` and a `Vec` of plain values). That changes when the writer adds types.
  - Section 4 relies on the `MutexGuard` being alive while the panic unwinds. The ADR should also forbid `Mutex::clear_poison` (stable since Rust 1.77, from my knowledge), because one call to it would silently remove the poison rule.
  - `try_lock` returns `Poisoned` only when it could take the lock. The export must map it to `MKPDF_ERR_INTERNAL` and drop the guard it got. `WouldBlock` maps to `MKPDF_ERR_BUSY`. These are from my knowledge of `std`, not run.
- Disposition: amend section 2 (`new` order), section 3 (guard scope for the `Arc` clone and the lock guard) and section 4 (no `clear_poison`). Add a test where a call holds the last clone during a concurrent free (test 10 already exists; make sure the drop happens inside `guard`). Owner: architect, rust-core. Date: before acceptance of ADR-0013.

### SEC-ADR-04 (low): lock order is sound; residual re-entry and `fork` hazards

- Evidence: `0013:59`, `0013:65-72`, the thread-local error state at `crates/mkpdf-ffi/src/lib.rs:81-101`.
- Analysis, all by reading:
  - One lock order exists: the table lock is taken, released, and only then is the document lock tried. The two are never held together. `try_lock` never waits, so a document lock cannot deadlock at all, even on the same thread.
  - The thread-local error state is a `RefCell` used with `try_borrow_mut` and `try_with`. It is not a lock, it is per thread, and it is touched after the closure returns, with no table or document lock held. No deadlock with it is possible.
  - Same-thread re-entry: no callbacks exist (ADR-0012 rejects them). The only remaining paths are (a) a Rust host that links the `rlib` or `staticlib` and installs a panic hook that calls an mkpdf function: the hook runs before unwinding, possibly with the table lock held, and `Mutex::lock` from the same thread may deadlock or panic (unspecified in `std`); (b) a call from a signal handler, which is not async-signal-safe; (c) `fork()` in a multi-threaded host while another thread holds the table lock, after which every call in the child blocks for ever.
  - All three are host bugs or edge cases, and none is UB in our code. They need one sentence each in `docs/api/c-abi.md`: no calls from signal handlers or panic hooks, and after `fork` only `exec`.
- Disposition: document. No change to the design. Owner: interop (c-abi.md). Date: with FFI-003.

### SEC-ADR-05 (medium): `MAX_LIVE_HANDLES` bounds the table, not memory

- Evidence: `0013:54`, `0013:233`, `crates/mkpdf-core/src/model.rs:8,12` (`MAX_PAGES = 100_000`, `MAX_TITLE_BYTES = 4096`), ADR-0012 rule 6 (`0012:64`).
- Analysis:
  - The table cost is small: about 16 bytes per slot plus 4 bytes per free-list entry (estimated). An empty document is about 100 bytes (estimated). The handle count is not where memory goes.
  - What one handle can hold is: today about 1.6 MB for a full document (100 000 pages of 16 bytes) and, with ADR-0012, up to `MAX_OUTPUT_BYTES` for each live buffer. With 65 536 buffers at, for example, 1 GiB each, the per-handle limits allow 64 TiB. So the product of the limits is not a bound. The real outcome of exhaustion is allocation failure, which aborts the host (`0011:18`, T-04).
  - A per-handle count is still useful as a leak detector. A wrapper whose finalizer never runs fills the table and then gets `LIMIT_EXCEEDED` for every new handle in the process. That is a graceful denial of service, and it affects every user of the library in that process, because the table is global.
- Recommended changes:
  - Add an aggregate budget for live buffer bytes (`MAX_LIVE_BUFFER_BYTES`), checked under the buffer table lock when a buffer is registered, released when it is freed. `save` returns `LIMIT_EXCEEDED` past it, before it allocates the result where it can (SEC-ADR-12).
  - Do not try to bound total document memory in the library now. Document it for hosts: memory per document is bounded by the core limits, and the total is the host's concern.
  - Grow the table with `try_reserve` so that table growth fails with `LIMIT_EXCEEDED` instead of aborting. This is cheap and removes one abort path.
- Disposition: ADR-0012 and ADR-0013 amendments. The owner sets the value of `MAX_LIVE_BUFFER_BYTES` with the writer. Owner: architect, owner (value). Date: before FFI-001.

## Findings in detail: ADR-0011 (panic strategy)

The choice of `unwind` is right for in-process hosts. One bad document becomes a status, not a crashed server (`0011:36`). I agree with rejecting alternatives A to D. The findings below are about whether the pin and the checks really hold.

### SEC-ADR-06 (medium): the `compile_error!` condition is negative, and the panic test must run on the release artefact

- Evidence: `0011:24-26`. Item 2 proposes `#[cfg(all(panic = "abort", not(target_arch = "wasm32")))]`. Item 3 proposes a CI check on the manifest and a panic test. `0011:18` says Cargo ignores the `panic` key for the `test` profile.
- Analysis:
  - The condition names the one bad strategy. A new strategy (for example the unstable `immediate-abort`) would pass it. Use the positive form: `#[cfg(not(panic = "unwind"))] compile_error!(...)`.
  - The `wasm32` exclusion is an exception for a build that does not exist. If the FFI crate is ever built for wasm, the exclusion makes the guard silently dead there. Drop it. A future wasm FFI build then fails to compile until someone decides, which is the safe default.
  - Profiles in the workspace `Cargo.toml` apply only when this workspace is the root of the build. A downstream crate that depends on `mkpdf-ffi` uses its own profile (Cargo behaviour from my knowledge, not run). So item 1 pins only our own artefacts. The `compile_error!` in item 2 is the control that actually reaches a downstream build. The ADR should say so, and the manifest check in item 3 is then a secondary check, not the main one.
  - Because `cargo test` always unwinds, a unit test cannot show that the shipped artefact unwinds. The test in item 3 must build the `cdylib` in the release profile and call a real export from a separate program (C or a Rust binary that loads it). That program can be the C caller test that SEC-F-15 and FFI-CR-10 already ask for.
- Disposition: amend items 2 and 3. Owner: architect (ADR), devops (CI job). Date: before acceptance of ADR-0011.

### SEC-ADR-07 (low): a test-only panic hook behind a Cargo feature can leak into a shipped build

- Evidence: `0011:26` and `0013:155` ("behind a feature never built in release").
- Analysis: Cargo features are additive. Any dependent, `--all-features`, or a packager can turn one on, and nothing stops a release build from having it. An exported "panic now" function in a shipped library is a denial-of-service primitive, and a trigger for any panic-path bug.
- Recommended: gate the hook with a custom `--cfg mkpdf_test_hooks` (set through `RUSTFLAGS` only in the test job, declared in `[lints.rust] unexpected_cfgs` so the lint stays clean). Add a release-pipeline check that the shipped `cdylib` exports exactly the symbols in the header (`nm`), so that no test symbol can ship. Better still, make the hook a non-exported function called from a test-only export that exists only under the cfg.
- Disposition: amend ADR-0011 item 3 and ADR-0013 test 11. Owner: architect, devops. Date: with the panic test.

### SEC-ADR-08 (medium): abort paths and side effects not covered by `guard` are not all listed

- Evidence: `0011:18`, `0011:28`, `crates/mkpdf-ffi/src/lib.rs:103-118` (`guard` drops the panic payload, then calls `set_last_error` outside `catch_unwind`), FFI-CR-06 (`ffi-code-review-architect.md:155-161`).
- Analysis. The ADR lists out-of-memory and stack overflow. It does not list:
  1. **A panic during unwinding** (for example a `Drop` that panics while a panic is unwinding) aborts the process. `catch_unwind` cannot stop it. With ADR-0013 the guards that drop during unwinding are a `MutexGuard` and an `Arc`, which do not panic. Core types must keep `Drop` panic-free. Add that rule to `rules/rust-engineering.md` through the architect.
  2. **A panic payload whose destructor panics.** `guard` drops the `Err(payload)` (`lib.rs:113`). A payload from `panic!` is a `String` or `&str` and is safe. A payload from `std::panic::panic_any` with a custom type could panic in `drop`, outside any `catch_unwind`. Not reachable today. Recommended: `std::mem::forget` the payload in `guard` (leak a few bytes per caught panic) or drop it inside a second `catch_unwind`.
  3. **Work after `catch_unwind`** (`failure.message()`, `set_last_error`) runs outside the guard. It cannot panic today except by allocation failure, which aborts anyway. Move it inside one `catch_unwind` as FFI-CR-06 proposes, with a static fallback text.
  4. **The default panic hook writes to the host's stderr.** The message includes the source path and the panic text. Our panic text must never contain caller data (T-27). Index values from a slice panic are caller-influenced numbers. Do not install a hook from the library (hostile to the host). Document the stderr output for SDK authors.
  5. **`extern "C-unwind"`.** The ADR proposes "never" (`0011:63`). Agree. Add a CI grep or a clippy rule so that it stays true.
  6. **The table mutex is recovered after poisoning** (`0013:60`). That is sound only if every table operation leaves the invariants intact at every point where a panic could happen. SEC-ADR-01 rule 3 (no overflow panic in `free`) is what makes this true.
- The facts in `0011:13-15` are verified by the ADR author on macOS arm64. I did not re-run them. The facts in `0011:18` are language facts that were not run here (the ADR says so). The abort since Rust 1.81 for unwinding out of `extern "C"` matches my knowledge.
- Disposition: amend the Consequences of ADR-0011 with points 1 to 5. Code changes for 2 and 3 with FFI-003. Owner: architect, rust-core. Date: before acceptance of ADR-0011.

### SEC-ADR-09 (low): the interim rule "validate first, mutate last" is not enforced

- Evidence: `0011:27`, T-20 (`threat-model.md:49`), SEC-F-03.
- Analysis: the interim rule holds today by inspection (`model.rs:87-115`: one assignment or one `push` after the check). It is enforced by nothing. If the writer or content streams become reachable through the FFI before FFI-003 lands, T-20 is open again with no poison rule.
- Disposition: the Orchestrator records that no new mutating FFI function merges before FFI-003 (D-08 already orders FFI-003 before FFI-001; this extends the rule to every mutating export). Owner: orchestrator. Date: now.

## Findings in detail: ADR-0012 (output buffers)

Option A is the right choice. It has no callback, no foreign unwinding, no size-query race and no allocator mismatch by design. I agree with rejecting B and C for the reasons given (`0012:32`, `0012:39`).

### SEC-ADR-10 (medium): the ABI sketch still uses pointer types, which contradicts D-13

- Evidence: `0012:48-55` declares `typedef struct MkpdfBuffer MkpdfBuffer;`, `mkpdf_document_save(const MkpdfDocument *doc, MkpdfBuffer **out)`, `mkpdf_buffer_data(const MkpdfBuffer *buf)` and `void mkpdf_buffer_free(MkpdfBuffer *buf)`. Rule 1 (`0012:59`) says a stale or foreign buffer pointer returns `NULL`, `0` or a no-op. ADR-0013 section 7 (`0013:106-112`) and D-13 make both handles `uint64_t` tokens and make `free` return a status.
- Analysis: as written, ADR-0012 cannot keep rule 1. A pointer cannot be checked without reading through it (SEC-F-04). If the owner accepted ADR-0012 alone, the text would promise a check that is impossible. ADR-0013 fixes this, but only in its own text.
- Disposition: amend the ADR-0012 sketch before acceptance, to tokens: `int32_t mkpdf_document_save(MkpdfDocument doc, MkpdfBuffer *out)` with `typedef uint64_t MkpdfBuffer;`, `int32_t mkpdf_buffer_free(MkpdfBuffer buf)`, and the accessor in SEC-ADR-11. State that ADR-0012 depends on ADR-0013. Owner: architect. Date: before acceptance of ADR-0012.

### SEC-ADR-11 (medium): data pointer lifetime under D-18, and `data` and `len` as two calls

- Evidence: `0012:52-53`, `0012:60-61` (rules 2 and 3), `0013:72`, `0013:91` (D-18: free succeeds while a call runs), `0013:110`.
- Analysis:
  - With D-18, `mkpdf_buffer_free` on thread B while thread A is inside `mkpdf_buffer_data` is allowed by the table. A clones the `Arc`, takes the pointer, and drops its clone on return. If B's free ran in between, A's clone is the last one, so the `Vec` is dropped **before A's caller receives the pointer**. A then returns a dangling pointer. The table makes the call itself safe, but the value it returns is already invalid. Rule 3 in ADR-0012 ("free must not run concurrently with any other call on the same buffer") forbids this, but D-18 says free is allowed on a busy handle. For buffers these two rules conflict, and the ADR must say which wins: for buffers, the rule is that the returned pointer is valid only until `mkpdf_buffer_free` **starts** on any thread, and the caller must order its own free after its last read.
  - `data` and `len` are two separate calls. Between them another thread can free the buffer, and `len` then returns `0` while the caller holds a pointer that is no longer valid, or a new buffer gets the same slot and a different length. The caller can mix up a pointer and a length from different states. Recommended: one accessor that returns both in one lookup, for example `int32_t mkpdf_buffer_view(MkpdfBuffer buf, const uint8_t **out_data, size_t *out_len)`, which writes `NULL` and `0` first (D-14) and returns `INVALID_HANDLE` for a bad token. A status-returning accessor also tells the caller **why** it got `NULL`, which the `NULL`/`0` convention in rule 1 does not.
  - Store the bytes as `Box<[u8]>` (or `Arc<[u8]>`), not `Vec<u8>`, so the buffer cannot grow, the pointer never moves, and no spare capacity is kept.
  - The pointer is `const`. A caller that writes through it while another thread reads it causes a data race in the caller's process. State it in the header.
  - This is the one place where memory safety still rests on the caller after ADR-0013 (the ADR says so at `0013:110`, which is accurate). It is the same class as T-02 (dangling pointers), not T-03.
- Disposition: amend ADR-0012 rules 2, 3 and the sketch. Add a test where a buffer is freed and the old token is passed to the accessor (`INVALID_HANDLE`, `NULL`, `0`), and a Miri test that reads the data while another thread frees only after a join. Owner: architect, rust-core, qa. Date: before acceptance of ADR-0012.

### SEC-ADR-12 (medium): `MAX_OUTPUT_BYTES` is unset, wrapper limits are not tied to it, and OOM aborts

- Evidence: `0012:64` (rule 6, value open), `0012:96`, wrapper rows at `0012:74-79` (Go `C.int` above 2 GiB, JNI `jint`).
- Analysis:
  - Several wrappers cannot hold more than about 2^31 bytes in one managed array: Go `C.GoBytes` takes a `C.int`, JNI arrays use `jint`, .NET `byte[]` is limited to about 2^31 elements, and Node `Buffer` has a version-dependent maximum. (These limits are from my knowledge, not checked, see "Claims".) If `MAX_OUTPUT_BYTES` is above them, each wrapper needs its own truncation-safe path, which is exactly where SEC-F-01-class bugs come from.
  - Recommended: `MAX_OUTPUT_BYTES` at most `2^31 - 1` bytes, so that the length fits in every signed 32-bit length. Propose **1 GiB (2^30)** as the first value, tested at the boundary, and documented as a limit that may only rise by a recorded decision.
  - "Allocation failure aborts the process in Rust" (`0012:64`) is true for the normal growing `Vec`. It is avoidable: the serialiser can count bytes and stop at the limit before it grows the buffer past it, and the final buffer can be reserved with `Vec::try_reserve_exact` (stable). With both, an over-limit document returns `LIMIT_EXCEEDED` and a failed reservation returns a status, instead of aborting the host. Growth during serialisation still uses infallible pushes unless the writer is built around a fallible sink. Document what remains.
  - Peak memory per `save` is the document plus the buffer, plus the wrapper copy (`0012:83`). The aggregate budget in SEC-ADR-05 bounds the live buffers in total.
- Disposition: set the value in ADR-0012 before FFI-001. Test at `MAX_OUTPUT_BYTES` and one past it (this needs a test-only way to lower the limit, under the cfg in SEC-ADR-07). Owner: architect (ADR), owner (value), rust-core. Date: before FFI-001.

## Findings in detail: cross-cutting

### SEC-ADR-13 (low): claims of closure and safety without a test

- Evidence: `0011:54` ("Panic containment (threat T-01) becomes a tested property"), `0012:91` ("T-23 is closed by rules 1, 2 and 4"), `0013:211` ("Memory safety for handles no longer depends on the caller"), `0013:165` and the table at `0013:199-207`.
- Analysis: each of these is a statement about code that does not exist. ADR-0013 itself says so (`0013:159`, `0013:238-241`), which I credit. ADR-0011 and ADR-0012 do not. T-23 in particular is not closed by rule 4: rule 4 is documentation. A caller can still call `free()` on the data pointer. Only the opaque token makes the mismatch impossible for the **buffer object**. For the data pointer it remains a caller contract.
- Disposition: reword to "will be closed when ..., evidence: tests N". See "Threat model coverage". Owner: architect. Date: with the amendments.

### SEC-ADR-14 (low): a valid token of another live object; Node `Number` precision

- Evidence: `0013:167`, `0013:215`.
- Analysis: the table cannot detect a valid token of another live object of the same kind (the ADR says so). One wrapper bug makes that likely: in Node, a token held in a `Number` is rounded once the generation reaches 2^21. Rounding changes the low bits, which are the index. The rounded value can then name a different slot that happens to have the same generation, which is another live document, so the call silently changes the wrong document (a confidentiality and integrity issue across requests in a server). Generation 2^21 is reached after about two million frees of one slot, which a long-running server does. In Java, a `long` token is negative when bit 63 is set (generation at or above 2^31), so a check such as `handle > 0` breaks. Wrappers must test with `!= 0`.
- Disposition: binding rules in `docs/api/c-abi.md`: Node uses `BigInt` only, every wrapper checks a token with `!= 0` and never with `> 0`. Binding tests start the table at a high generation (test-only constructor) to catch both bugs. Owner: interop. Date: before the first SDK.

### SEC-ADR-15 (low): `BUSY` on read-only calls and on concurrent `save`

- Evidence: `0013:65-70`, `0012:63` (rule 5: `save` takes `const`), `0012:61`.
- Analysis: every document call, including `page_count` and `save`, takes the document mutex with `try_lock`. So two concurrent reads of one document return `BUSY`. That matches the one-thread-at-a-time contract (D-15), but ADR-0012 rule 5 reads as if `save` were freely shareable. The ADRs should say that `save` is repeatable and does not change the document, but it is still a call on the handle and may return `BUSY`. Buffers take no mutex and never return `BUSY` (`0013:108`), which is correct. A wrapper that answers `BUSY` with a spin loop turns a caller bug into a CPU-bound hang. Wrappers must surface it as an error.
- Disposition: one sentence in each ADR and in `c-abi.md`. Owner: architect, interop. Date: with the amendments.

## Proposed MAX_LIVE_HANDLES (D-19)

**Proposal: `MAX_LIVE_HANDLES = 65 536` (`1 << 16`) live handles per kind**, for documents and for buffers separately, plus the hard slot cap of `1 << 24` from SEC-ADR-01 rule 1 and the aggregate buffer byte budget from SEC-ADR-05. This is a proposal. The owner accepts or changes it.

Reasoning:

1. **What the limit is for.** It does not bound memory (SEC-ADR-05). It bounds the table, and it turns a handle leak in a wrapper (a finalizer that never runs, a missing `Dispose`) into `LIMIT_EXCEEDED` before the leak turns into an out-of-memory abort of the host. A lower limit makes that failure appear sooner and with less memory held.
2. **Legitimate use.** A handle is an in-memory document or one serialised output. A server that builds documents per request holds about one document and at most one buffer per request in flight. 65 536 concurrent documents in one process is far above any realistic concurrency for an in-memory engine. I found no stated use case in the repository that needs more. If one appears, raising the limit is a behaviour change that needs a recorded decision and a minor version, not a redesign.
3. **Cost at the limit.** Table: about 65 536 x 16 bytes, about 1 MiB per kind, plus the free list (estimated, not measured). Documents at the limit: about 6.5 MB if empty (estimated, about 100 bytes each). The table never dominates.
4. **Distance from the index field.** 2^16 is 256 times below the 2^24 index space. Together with the slot cap, a truncated index can never be issued (SEC-ADR-01), and about 16 million slot retirements fit before the slot cap is reached.
5. **Why not 2^20 (FFI-CR-05).** 2^20 costs about 16 MiB of table at the limit and lets a leak hold sixteen times more memory before it is reported. It gives no benefit for a real use case. The difference is not a safety issue, so 2^20 is an acceptable owner choice if preferred, with the same tests.
6. **The table is global per process.** All users of the library in one process share it. One leaking component exhausts it for all others. That is a reason to make the limit visible: document it in `c-abi.md`, return a distinct error text ("live document handle limit reached"), and add a counter that a host can read later (not proposed now).

Tests required for the value: with the real constant, a test that the constant equals `1 << 16` and is below `1 << 24` (a `const` assert). With a small test table (test-only constructor, ADR-0013 test 7): the one-past `new` returns `LIMIT_EXCEEDED`, writes `0`, and a `free` makes room again. A slot-cap test with retired slots (SEC-ADR-01).

## Is ADR-0013 sound enough to accept?

**Yes, with changes.** On the questions asked:

| Question | Answer | Basis |
|---|---|---|
| Can a token value cause UB? | No. No value is dereferenced. Decoding is integer arithmetic | Design, by reading (`0013:33-43`, `0013:58`) |
| Out-of-bounds index? | No, if lookup uses `get` and the slot count is capped at 2^24 | SEC-ADR-01 rules 1 and 2 (to be added) |
| Panic from a token? | No, if the generation bump is checked. Otherwise yes, in `free` at `u32::MAX` | SEC-ADR-01 rule 3 (to be added) |
| Unbounded allocation? | No from a token. Table growth is bounded by the limit and the slot cap. Memory per handle is bounded by core limits; total buffer memory needs SEC-ADR-05 | SEC-ADR-05 |
| Slot retirement and wraparound | Sound. Retired slots are never reused, so no generation repeats in a slot. The case is reachable by a plain loop, so test it | SEC-ADR-02 |
| Lock order and deadlock | One lock order, locks never nested, `try_lock` never waits. No deadlock with the thread-local error state, which is not a lock | SEC-ADR-04 |
| Poisoning | Sound for the document mutex if guards live inside `catch_unwind` and `clear_poison` is never called. Table recovery with `into_inner` is sound if table steps cannot panic half-way | SEC-ADR-03, SEC-ADR-08 item 6 |
| Free on poisoned or busy handle (D-18) | Sound for documents: the running call keeps its `Arc` clone. For buffers it conflicts with ADR-0012 rule 3 and needs the wording in SEC-ADR-11 | SEC-ADR-11 |
| Reentrancy | None through the ABI (no callbacks). Panic hook, signal handler and `fork` need documentation | SEC-ADR-04 |

The changes needed before the owner accepts it: SEC-ADR-01 (rules 1 to 4 in the text), SEC-ADR-03 (allocation order and guard scope), and the value of `MAX_LIVE_HANDLES`. SEC-ADR-02, 04, 05, 14 and 15 can be amended in the same commit, or tracked as low findings with an owner and date. None of this changes the D-11 design, and the D-13 to D-18 decisions all stand.

## Threat model coverage

No threat below may be marked "Tested" until the named tests exist and pass, including under Miri where stated. Today none of these tests exists.

| Threat | Effect of the ADRs when implemented | Evidence needed before the status may change | Status today |
|---|---|---|---|
| T-01 panic across the FFI | ADR-0011 pins `unwind` and adds a compile-time check. Containment per export becomes testable | `compile_error!` present; CI job that builds with `-C panic=abort` and expects failure; release-artefact panic test through a real export (SEC-ADR-06) | Unchanged: tested for `guard` only |
| T-03 double free, use after free | ADR-0013 makes both an error status for handles | ADR-0013 tests 2, 3, 4, 5, 6, 10 green under Miri | Open (design decided, D-08/D-11) |
| T-07 data race on a shared handle | ADR-0013 replaces UB with `BUSY` | Tests 8 and 9 under Miri with threads | Documented, not enforced |
| T-20 handle after a caught panic | ADR-0013 section 4 via mutex poisoning | Test 11 through a real export | Open |
| T-22 out parameters on failure | D-14 neutral values | Test 12 per function | Open |
| T-23 buffer allocator mismatch | ADR-0012: opaque buffer, library free. The data pointer remains a caller contract (SEC-ADR-13) | Buffer tests in ADR-0012 Consequences; a header statement | Planned. Not closed by ADR-0012 alone |
| T-24 reentrancy through callbacks | Does not arise: ADR-0012 has no callbacks | n/a until a streaming ADR | Planned (deferred) |
| T-04 resource exhaustion | Handle limit, slot cap, `MAX_OUTPUT_BYTES`, aggregate buffer budget, `try_reserve` | Boundary tests for each (SEC-ADR-05, 12) | Partly mitigated |
| T-02 dangling pointers | Narrowed: handles are no longer pointers. Out pointers, string pointers and the buffer data pointer remain | n/a: cannot be detected | Partly mitigated |
| T-21 error text on thread migration | Unchanged by these ADRs | SDK pinning tests | Open |
| T-27 content in panic output | Panic hook writes to host stderr (SEC-ADR-08 item 4) | Rule and review of panic messages | Documented |

New hazards these ADRs introduce, which the threat model should gain when the owner accepts them: global handle table exhaustion by one leaking component (SEC-ADR-05, §6 above), wrong-object use through a corrupted token (SEC-ADR-14), and `fork` or signal-handler calls with the table lock held (SEC-ADR-04). I have not edited `docs/security/threat-model.md` in this review. That update should be made when the ADRs are accepted, so that the model describes decided designs.

## Claims about wrapper and language behaviour

Verified = checked by someone with a recorded run. Unverified = stated without a run here. Assumed = a design property of code that does not exist.

| Claim | Where | State |
|---|---|---|
| `catch_unwind` exits 0 with `unwind`, 134 with `abort`; `compile_error!` on `cfg(panic)` works on stable; release sizes | `0011:13-15` | Verified by the ADR author (macOS arm64). Not re-run by me |
| Unwinding out of `extern "C"` aborts since Rust 1.81; stack overflow and OOM abort; Cargo ignores `panic` for `test`; no unwinding on `wasm32-unknown-unknown` | `0011:18` | Unverified here. Consistent with my knowledge |
| Workspace profile pins downstream builds | implied by `0011:24` | Unverified, and from my knowledge false: profiles apply only at the build root (SEC-ADR-06) |
| Mutex is poisoned when the guard drops during unwinding; `try_lock` gives `WouldBlock` or `Poisoned` | `0013:77`, `0013:239` | Unverified here. Consistent with my knowledge of `std` |
| Miri supports the threaded tests | `0013:153` | Unverified. Miri has never run in this repository |
| Go `C.GoBytes` takes `C.int`; goroutines migrate OS threads; Go pointers must not be kept by C | `0012:74` | Unverified here. Consistent with my knowledge |
| `ctypes` default `int` return; `__del__` at shutdown | `0012:75` | Unverified here |
| Electron V8 memory cage restricts external buffers; `Number` exact to 2^53 | `0012:76`, `0013:167` | Unverified here |
| `SafeHandle` makes double free harmless; `DangerousAddRef` for spans | `0012:77` | Unverified here |
| `NewDirectByteBuffer` outlives free; FFM `reinterpret` with cleanup (Java 22+) | `0012:78` | Unverified here |
| `unsafe` block count 8 today, 2 after ADR-0013 | `0013:13`, `0013:136` | 8 verified by the architect at `a57d899` (FFI-CR-15). 2 is assumed (design estimate, the ADR says so) |
| `free` needs no `unsafe` | `0013:93` | Assumed. Plausible from the design |
| A forged value "fails on the index, the kind or the generation" | `0013:58` | Assumed, true only with SEC-ADR-01 rules 1 to 4 |
| T-23 closed by rules 1, 2, 4 | `0012:91` | Assumed, and overstated (SEC-ADR-13) |
| Panic containment "becomes a tested property" | `0011:54` | Assumed. No such test exists |

## Owners and dates (proposed)

Medium and low findings need an owner and a date. These are proposals for the Orchestrator to confirm. Unless stated otherwise, the date is "before the owner accepts the ADR concerned", and no later than 2026-12-14.

| Finding | Owner | Date |
|---|---|---|
| SEC-ADR-01, 02, 03 | architect (text), rust-core (code), qa (tests) | Before ADR-0013 acceptance |
| SEC-ADR-04 | interop | With FFI-003 |
| SEC-ADR-05 | architect; owner sets the byte budget | Before FFI-001 |
| SEC-ADR-06, 07, 08 | architect, devops, rust-core | Before ADR-0011 acceptance |
| SEC-ADR-09 | orchestrator | Now |
| SEC-ADR-10, 11 | architect | Before ADR-0012 acceptance |
| SEC-ADR-12 | architect; owner sets `MAX_OUTPUT_BYTES` | Before FFI-001 |
| SEC-ADR-13 | architect | With the amendments |
| SEC-ADR-14, 15 | interop, architect | Before the first SDK |

## Statement on acceptance and risk

This review accepts no ADR and no risk. Recommendations above are inputs for the owner. No finding is resolved, because no implementation exists. Findings move to "Resolved" only in a later revision of this file, with the commit and the test or run evidence named. Critical and high findings: none, so nothing needs immediate escalation. The medium findings against ADR-0013 (SEC-ADR-01, 03) should be in the text before the owner accepts it, because they are cheap to state and expensive to discover after the ABI change.

## Next review

Review again when the ADRs are amended, when FFI-003 code exists (the table module, every export, and the tests in ADR-0013 section 10, with a Miri log), and when the writer sets `MAX_OUTPUT_BYTES`.

## Re-review after amendments (2026-11-24)

Reviewer: Security Engineer. Scope: the amended ADR-0011, ADR-0012 and ADR-0013 at commit `ffa10a6`, and the interop changes in `4a0751b` (`crates/mkpdf-ffi/src/lib.rs`, `include/mkpdf.h`, `docs/api/c-abi.md`, `docs/api/rust-api.md`, `CHANGELOG.md`). Line references are `0011:N`, `0012:N`, `0013:N` for `docs/adr/001N-*.md` at `ffa10a6`. This section accepts no ADR and no risk.

### What was run (2026-11-24, macOS arm64, local, rustc 1.100.0 Homebrew, HEAD `ffa10a6`, clean tree)

| Command | Result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` (after `touch` on both crate sources, so the crates were re-checked) | exit 0, `Finished`, no warnings |
| `cargo test --workspace --locked` | exit 0. `mkpdf-core` 8 passed, `mkpdf-ffi` 15 passed, `mkpdf-cli` 0, doc-tests 0 and 0. 23 tests in total, 0 failed, 0 ignored |
| `cargo deny check` | exit 0: `advisories ok, bans ok, licenses ok, sources ok`. One warning: `Zlib` is an unmatched licence allowance. The workspace has no third-party dependency, so this checks almost nothing |
| `gh run view` on runs 45103722429 (`ccc73d5`) and 45325022117 (`9acbb60`) | Both `success` on all 10 jobs. The `miri-ffi` log of 45103722429 lists 12 tests passed, including the three `set_title_rejects_len_*` tests, under `rustc 1.102.0-nightly` |
| Stand-alone check of the positive `cfg` form (ADR-0011 item 2) | `#[cfg(not(panic = "unwind"))] compile_error!` builds with `-C panic=unwind` (exit 0) and fails with `-C panic=abort` (exit 1). This confirms `0011:39` |
| Stand-alone check of `Arc<[u8]>` storage (ADR-0012 rule 3), `rustc -O` | `Arc::<[u8]>::from(vec)` puts the bytes in a **new** allocation (data pointer differs from the `Vec` pointer). `Arc::new(vec.into_boxed_slice())` keeps the original allocation. `size_of::<Option<Arc<[u8]>>>()` = 16, `size_of::<Option<Arc<Box<[u8]>>>>()` = 8. See SEC-ADR-16 |

### What was NOT run

- Miri locally (no rustup, no nightly, `cargo-miri` not installed). The Miri evidence is from CI only, and only up to `9acbb60`.
- `cargo audit` (not installed locally). CI `audit` job passed in both runs above.
- **Commits `4a0751b` and `ffa10a6` are local and not pushed.** The 15-test FFI suite, including the three tests added in `4a0751b`, has not run in CI or under Miri. `4a0751b` changes no executable line in `lib.rs` (its removed lines are all comments), so the Miri result for the `set_title` code still applies to the code. It does not cover the new tests.
- `cargo test --release`, Linux, Windows, the `wasm32` target (not installed, so a 32-bit build of the proposed constants was not tried), a `-C panic=abort` build of `mkpdf-ffi` itself, and any C caller.
- None of the ADR designs is implemented, so every "resolved" below means **the text now says the right thing**. Nothing in ADR-0011 to ADR-0013 has been tested.

### Task 1: status of each finding

| ID | Sev | Status | Evidence | Remaining |
|---|---|---|---|---|
| SEC-ADR-01 | medium | **Resolved (text)** | Rules 1 to 4 at `0013:107-112`; constants and `const` assert at `0013:102-104`; checked `u32` index, no `as` truncation at `0013:92`; order in `free` at `0013:111` and `0013:149`; lookup order (kind, index, generation, value) at `0013:112`; tests 5, 6, 8 at `0013:230-233` | Implementation and tests (FFI-003b) |
| SEC-ADR-02 | low | **Resolved** | "minutes to tens of minutes" at `0013:150` (2^32 cycles at a few million per second is about 20 to 70 minutes); retirement is a required test at `0013:150` and `0013:231` | None. See the disagreement below |
| SEC-ADR-03 | medium | **Resolved (text)** | Allocation before the table lock and drop after it at `0013:116`; `Arc` clone and guard inside `catch_unwind` at `0013:137` and `0011:73`; `clear_poison` forbidden at `0013:140` and `0011:70`; test 11 with a panicking `Drop` at `0013:236` | The `disallowed-methods` entry for `clear_poison` is proposed, not written (rust-core or devops) |
| SEC-ADR-04 | low | **Partly resolved** | Signal handler, panic hook and `fork` listed at `0013:204-212` | The sentence in `docs/api/c-abi.md` is deferred to interop with FFI-003b (`0013:38`). I accept the deferral, because the ABI it describes does not exist yet |
| SEC-ADR-05 | medium | **Resolved (text), value open** | `try_reserve` for table growth at `0013:116`; byte budget at `0012:96` and `0013:175`; "the handle count bounds the table, not memory" at `0012:96` and `0013:304` | Owner sets the budget. See SEC-ADR-17 for two gaps in the budget rule |
| SEC-ADR-06 | medium | **Resolved (text)** | Positive form, no `wasm32` exception at `0011:63`; the profile pin described as secondary at `0011:62`; release-`cdylib` panic test through a real export at `0011:67`; expected-failure CI build at `0011:66`. Positive form re-run by me (see above) | CI jobs and the test (devops, rust-core). Condition C-2 stays open |
| SEC-ADR-07 | low | **Resolved (text)** | `--cfg mkpdf_test_hooks`, declared in `unexpected_cfgs`, not a Cargo feature at `0011:68`; symbol check against the header at `0011:69`; the same gate for ADR-0013 tests 6 and 12 and ADR-0012 test 4 (`0013:231`, `0013:237`, `0012:126`) | Implementation |
| SEC-ADR-08 | medium | **Resolved (text)** | Abort paths listed at `0011:47-56`, including `panic_nounwind` and a panic during unwinding; payload `mem::forget` or nested `catch_unwind`, and `set_last_error` inside the guard, at `0011:73`; panic-free `Drop` at `0011:72` | The rules-file line (panic-free `Drop`) and the `C-unwind` CI check are proposals to the owner and devops (`0011:24-25`). I accept that the architect cannot edit the rules file. Both stay open with those owners |
| SEC-ADR-09 | low | **Partly resolved** | Sequencing rule stated at `0011:71` and marked as a proposal for the Orchestrator (`0011:26`) | `docs/roadmap.md` does not record it (no entry mentions mutating exports; `roadmap.md:55-56` covers only FFI-003a/b). No new mutating export has been added since the first review (`4a0751b` changes only the existing `set_title`), so nothing has been broken yet. Orchestrator to record it |
| SEC-ADR-10 | medium | **Resolved** | `typedef uint64_t MkpdfBuffer` and status-returning accessors at `0012:72-84`; dependency on ADR-0013 at `0012:5` | None |
| SEC-ADR-11 | medium | **Partly resolved** | Single `mkpdf_buffer_view` at `0012:79` and `0012:89`; pointer valid until `free` **starts** at `0012:89`, `0012:91`, `0013:172`; buffers never `BUSY` at `0012:90`; proposed amendment to `rules/interoperability.md` item 2 at `0012:135` | The storage type `Arc<[u8]>` copies the whole output on construction (new finding SEC-ADR-16). `0013:170` says "held as a boxed slice", which is not what `Arc<[u8]>` is |
| SEC-ADR-12 | medium | **Partly resolved, value open** | `1 << 30` proposed, not decided, at `0012:94` and `0012:143`; `try_reserve_exact` at `0012:95`; writer caveat stated at `0012:95` | Owner sets the value. Writer enforcement waits for PDF-001. `try_reserve_exact` is undone by the `Arc<[u8]>` conversion (SEC-ADR-16) |
| SEC-ADR-13 | low | **Resolved** | "will be a tested property ... once" at `0011:105`; "will be closed when FFI-003b is implemented and tests 1 to 3 pass" at `0012:134`; "None exists yet" at `0013:224`; counts called estimates at `0013:199` and `0013:296`; the Miri statement re-based on CI evidence at `0013:248`, which I checked against the job log | None |
| SEC-ADR-14 | low | **Partly resolved** | Wrong-object hazard, Node `BigInt` only, `!= 0`, Java sign bit, high-generation test constructor at `0013:214-220` and `0013:231`; wrapper table at `0012:102` and `0012:110-112` | The rules go into `docs/api/c-abi.md` (interop) before the first SDK, as the first review proposed. Not a blocker for acceptance |
| SEC-ADR-15 | low | **Resolved (text)** | `BUSY` on every document call including `page_count` and `save` at `0013:127`; no silent retry at `0013:129` and `0013:164`; `save` may return `BUSY` at `0012:93`; test 9 covers a read-only call and a second `save` at `0013:234`; ADR-0012 test 6 at `0012:128` | The `c-abi.md` sentence comes with FFI-003b (interop) |

### Task 1: the architect's disagreements

**SEC-ADR-02, LIFO kept (`0013:37`). I accept the reasoning.** My finding asked for the wording to be fixed and said FIFO was optional. Retirement already removes ABA reuse: a retired slot has `value = None` and generation `u32::MAX` for good (`0013:112`), so FIFO would only spread retirement over more slots and gives no safety gain. Even with LIFO the slot cap is out of reach: about 2^24 retirements times about 2^32 cycles each is about 2^56 frees. One correction to the cost figure at `0013:150`: "about 16 bytes" holds for the document table. For the buffer table as designed (`Option<Arc<[u8]>>` is 16 bytes, measured) a slot is about 24 bytes. That is still small.

**SEC-ADR-11, `Arc<[u8]>` instead of `Box<[u8]>` (`0012:28`). I accept the reason, not the type.** The architect is right that the table holds every value as an `Arc` so that D-18 (`free` during a running call) is safe. My `Box<[u8]>` proposal was about the bytes, not about the outer owner, and an `Arc` around them is correct. But `Arc<[u8]>` is not "a boxed slice inside the `Arc`" as `0012:28` says. It is one allocation that holds the reference counts and the bytes together, so building it from the writer's `Vec<u8>` allocates again and copies every byte. I measured this (see "What was run"). The fix is `Arc<Box<[u8]>>` or `Arc<Vec<u8>>`. See SEC-ADR-16.

**SEC-ADR-12, writer enforcement stated as a requirement (`0012:26`) and the value only proposed (`0012:27`). I accept both.** The writer does not exist, so the ADR cannot promise how it grows its output. `0012:95` now says what is left if growth cannot be made fallible: an allocation failure inside the writer aborts the host. That is honest, and it is the correct residual for the owner to see. The value is the owner's to set under D-20, and I made the proposal myself.

The other partial dispositions (`0011:24-27`, `0013:38-39`) move work to its owners (owner for rules files, devops for CI, interop for `c-abi.md`). I accept them. Each item stays open in the table below with its owner.

### New findings

#### SEC-ADR-16 (medium): `Arc<[u8]>` copies the whole output with an allocation that cannot fail gracefully

- Evidence: `0012:14`, `0012:28`, `0012:90` (rule 3), `0013:170`. Stand-alone test (rustc 1.100.0, `-O`): `Arc::<[u8]>::from(Vec<u8>)` returns a data pointer that differs from the `Vec`'s, so the bytes were copied into a new allocation. `Arc::new(vec.into_boxed_slice())` keeps the same pointer. This matches the standard library design: `Arc<[T]>` stores the counts and the data in one block, so it cannot adopt the `Vec`'s buffer.
- Problem: (1) The copy allocates up to `MAX_OUTPUT_BYTES` (1 GiB as proposed) through the normal, infallible path. Rule 8 (`0012:95`) reserves the final buffer with `try_reserve_exact` so that an out-of-memory condition returns a status. The `Arc<[u8]>` conversion right after it allocates the same size again and **aborts** on failure, so rule 8 has no effect. (2) Peak memory during `save` doubles. That is not counted by the byte budget (SEC-ADR-17). (3) `0013:170` says the bytes are "held as a boxed slice", which describes a different type.
- Fix: store `Arc<Box<[u8]>>`, or `Arc<Vec<u8>>`. `Arc<Vec<u8>>` avoids every reallocation. `into_boxed_slice` may reallocate if the capacity exceeds the length, because `try_reserve_exact` may give more than asked. Spare capacity in an immutable `Vec` behind an `Arc` cannot grow or move the data, because no `&mut` exists while the buffer is shared. Either type keeps every property in rule 3. The `Arc` control block (a few dozen bytes) is still an infallible allocation, which is acceptable. Correct `0012:14`, `0012:28`, `0012:90` and `0013:170` to the chosen type. Add a test that the data pointer of the stored buffer equals the pointer of the writer's output, or simply that only one large allocation happens.
- Owner: architect (text, before ADR-0012 acceptance), rust-core (code, FFI-001). Due 2026-12-14.

#### SEC-ADR-17 (low): the byte budget rule leaves out in-flight memory and has no compile-time checks

- Evidence: `0012:96` ("checked ... when a buffer is registered", "before it allocates the result where it can"); `0012:144`.
- Problem: (1) The budget counts buffers that are already registered. A `save` serialises first and registers after, so the memory of serialisations in progress (and today the `Arc<[u8]>` copy) is outside the budget. With N concurrent `save` calls on different documents the peak is about the budget plus N times the output size. (2) Nothing ties the constants together: the budget must be at least `MAX_OUTPUT_BYTES` (or one maximum save can never succeed), and `MAX_OUTPUT_BYTES` must be at most `i32::MAX` (the reason given at `0012:94`). (3) `4 * (1 << 30)` does not fit a 32-bit `usize`. All Tier 1 targets are 64-bit (OQ-03) and the FFI crate is not built for `wasm32` (ADR-0009), so this is not reachable today. A `const` overflow is a compile error, not a wrap (from language knowledge; a 32-bit build was not run because the target is not installed).
- Fix: (1) Either reserve budget before serialising (take `MAX_OUTPUT_BYTES`, or an estimate, under the buffer table lock and settle on registration), or state in ADR-0012 and `c-abi.md` that the budget bounds retained buffers only. Either choice is acceptable. The second needs only text. (2) Add `const _: () = assert!(MAX_OUTPUT_BYTES <= i32::MAX as usize);` and `const _: () = assert!(MAX_LIVE_BUFFER_BYTES >= MAX_OUTPUT_BYTES as u64);`. (3) Keep the budget as `u64`, or document that 32-bit FFI targets are not supported.
- Owner: architect (text), rust-core (constants). Before FFI-001, no later than 2026-12-14.

### Task 2: proposed values

| Value | Proposed | My view | Reasoning |
|---|---|---|---|
| `MAX_LIVE_HANDLES` | 65 536 (`1 << 16`) per kind (D-20, `0013:102`) | **Agree** | This is my own proposal (D-19), now recorded by the owner. It turns a leaking wrapper into `LIMIT_EXCEEDED` long before the table is a memory problem. It does **not** bound document memory: one document can hold `MAX_PAGES` = 100 000 pages and a 4 KiB title, so 65 536 live documents can use far more than the table. `0013:304` says the total is the host's concern. That is a residual risk the owner should see. It is not a defect of this value |
| Slot cap | `1 << 24` (`0013:103`), `const` assert at `0013:104` | **Agree** | It equals the index space, so no index can be truncated (SEC-ADR-01 rule 1). Worst-case table size is estimated at 2^24 slots times 16 bytes (documents) = 256 MiB, and 24 bytes for buffers with `Arc<[u8]>` (16 with SEC-ADR-16 applied). Not measured. Reaching the cap takes about 2^56 frees (see SEC-ADR-02), so in practice the cap guards correctness, not memory. `Vec` growth may reserve up to twice the length in capacity. That is harmless, because `try_reserve` fails cleanly |
| `MAX_OUTPUT_BYTES` | `1 << 30` (`0012:94`) | **Agree** | 2^30 = 1 073 741 824 is below 2^31 − 1, so Go `C.int`, JNI `jint` and .NET array lengths cannot truncate it. Add the `i32::MAX` assertion (SEC-ADR-17). Not decided. The owner sets it |
| Live-buffer byte budget | `4 * MAX_OUTPUT_BYTES` = 4 GiB (`0012:96`) | **Agree as a starting value**, with SEC-ADR-17 | It allows four maximum-size buffers, or many small ones, and keeps 65 536 × 1 GiB = 64 TiB out of reach. It bounds retained buffers only, not in-flight serialisation (SEC-ADR-17). A server host may want it lower. A later setter that may only lower the value would be a safe addition. Not decided. The owner sets it |

### Task 3: checks

See "What was run" and "What was NOT run" above. In short: fmt, clippy and tests pass locally at `ffa10a6` (23 tests: 8 core, 15 FFI). `cargo deny check` passes. CI is green at `9acbb60` and earlier. The last two commits have not run in CI. Miri, `cargo audit`, release-mode tests, other platforms and any C caller were not run locally.

### Condition C-1 (from `docs/reviews/gate-1-security.md`)

C-1 asks for SEC-F-01 and SEC-F-02 fixed with tests, the commit and test names, and a review by security. The previous re-review (2026-11-23) said it needed the commit, SEC-F-20 fixed, and the commit hash recorded.

| Part | Evidence |
|---|---|
| SEC-F-01 fix committed | `eeed518` (`lib.rs:230-232` at HEAD: `len > isize::MAX` returns `INVALID_ARGUMENT` before the slice is built; compile-time check `MAX_TITLE_BYTES <= isize::MAX` at `lib.rs:37`). The test bodies are unchanged in `4a0751b` (`git diff eeed518 HEAD` removes comment lines only) |
| SEC-F-01 tests | `set_title_rejects_len_usize_max_without_reading`, `set_title_rejects_len_just_over_isize_max_without_reading`, `set_title_rejects_len_isize_max_as_limit_without_reading`, plus `set_title_null_utf8_with_huge_len_is_null_pointer` (new in `4a0751b`). All pass locally. The first three passed under Miri in CI run 45103722429, which checks the order of the checks in any build mode |
| SEC-F-02 fix committed | `eeed518` (`model.rs:11` constant, `model.rs:87` `set_title`, `lib.rs:233-234`); documentation in `4a0751b` (`c-abi.md:67-82`, `mkpdf.h:25-26` and `45-57`, `rust-api.md:13` and `30`, `CHANGELOG.md:16-22`) |
| SEC-F-02 tests | Core: `title_at_limit_is_accepted`, `title_over_limit_is_rejected_and_old_title_kept`, `title_limit_counts_bytes_not_chars`. FFI: `set_title_over_limit_is_rejected_and_keeps_previous_title`, `set_title_at_limit_is_accepted`, `set_title_failures_keep_previous_title`. All pass locally |
| SEC-F-20 | Fixed in `4a0751b` except two small items: the rustdoc of `MAX_TITLE_BYTES` still points to T-10 instead of T-04 (`model.rs:10-11`), and no test compares `MKPDF_MAX_TITLE_BYTES` in the header with the Rust constant. Threat model T-04 and T-19 are corrected by this review. Neither remaining item affects the limit or what callers are told |
| Review by security | This section |

**C-1 is met**, as of `4a0751b`, for the purpose it serves (FFI-001 may merge as far as C-1 is concerned). Caveats, none of which blocks it: `4a0751b` is not pushed and has not run in CI; allocation failure elsewhere still aborts (T-04 stays "Partly mitigated"); the release-mode order of checks is proven by Miri, not by `cargo test` (mutation M1). The SEC-F-01 and SEC-F-02 rows in the GATE-1 findings table still say "not committed". I was asked to change only SEC-F-19 to SEC-F-21 there, so those two rows are left for the owner to approve an update.

### Task 4: recommendations to the owner

| ADR | Recommendation | Conditions |
|---|---|---|
| ADR-0011 panic strategy | **Accept** | No text change needed for security. The required work (the `compile_error!`, the expected-failure `panic=abort` CI job, the release-`cdylib` panic test through a real export, the `nm` symbol check, `guard` scope per item 7) is implementation. It is tracked by C-2 and must be done before v0.1.0. The owner also decides on the two proposed rules-file lines (`0011:117`) |
| ADR-0012 output buffers | **Accept with changes** | (1) SEC-ADR-16: change the storage type and the four lines that name it. (2) SEC-ADR-17: state what the budget bounds and add the two `const` assertions. (3) The owner sets `MAX_OUTPUT_BYTES` and the budget (I support `1 << 30` and 4 GiB). It can only be accepted after ADR-0013 (`0012:5`) |
| ADR-0013 handles | **Accept with one change** | Correct `0013:170` ("`Arc<[u8]>` ... held as a boxed slice") to the type chosen for SEC-ADR-16. That is one sentence. Everything I asked for in SEC-ADR-01 to 05, 14 and 15 is in the text. Implementation (FFI-003b), the 17 tests in section 12 and a Miri run with threads are needed before T-03 may move to "Tested" |

### Open items after this re-review

| ID | Sev | What remains | Owner | Due |
|---|---|---|---|---|
| SEC-ADR-16 | medium | `Arc<[u8]>` copy undoes `try_reserve_exact`. Change the type in `0012` and `0013:170` | architect, rust-core | Before ADR-0012 acceptance; ≤ 2026-12-14 |
| SEC-ADR-17 | low | Budget scope (in-flight memory) and `const` assertions | architect, rust-core | Before FFI-001; ≤ 2026-12-14 |
| SEC-ADR-12 | medium | Owner sets `MAX_OUTPUT_BYTES`; the writer enforces it during growth | owner, PDF-001 | Before FFI-001 |
| SEC-ADR-05 | medium | Owner sets the byte budget | owner | Before FFI-001 |
| SEC-ADR-09 | low | Sequencing rule not in `docs/roadmap.md` | orchestrator | Now |
| SEC-ADR-04, 14, 15 | low | Sentences in `docs/api/c-abi.md` | interop | With FFI-003b; SEC-ADR-14 before the first SDK |
| SEC-ADR-03 | — | `clear_poison` `disallowed-methods` entry | rust-core or devops | With FFI-003b |
| SEC-ADR-06, 07, 08 | — | CI jobs, release-artefact test, symbol check, `C-unwind` check, two rules-file lines | devops, rust-core, owner | Before v0.1.0 (C-2) |
| All | — | No design is implemented or tested. Each finding marked "Resolved (text)" is re-checked against code and a Miri log when FFI-003b and FFI-001 exist | security | Next review |

### Risk acceptance

None. This re-review accepts no ADR and no risk, and none was asked for. No critical or high finding exists, so nothing is escalated.
