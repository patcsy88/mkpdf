# Code review: `mkpdf-ffi` at a57d899 (architect and Rust reviewer)

- Reviewer: Solution Architect (agent role). This is a code review. It is not the security gate. Under `rules/security.md` the Security Engineer must still approve any change to `mkpdf-ffi`, and this record does not replace that.
- Model: my configuration reports `claude-sonnet-5-5` (provider `cliproxyapi`). This is unverified, and I cannot check it from inside the session.
- Date: 2026-11-23
- Subject: `crates/mkpdf-ffi/src/lib.rs` (358 lines) and `crates/mkpdf-ffi/Cargo.toml` as committed at `a57d899`, read with `git show a57d899:<path>`. Line numbers below are lines of that file. The working-tree copy and the in-flight edits to `mkpdf-ffi` and `mkpdf-core` were not reviewed.
- Compared against: `include/mkpdf.h` (identical at `a57d899` and in the working tree, checked with `diff`), `docs/api/c-abi.md` (working tree, which includes my earlier edits), `rules/rust-engineering.md`, `rules/interoperability.md`, and the core API at `a57d899` (`model.rs` read only for the signatures and limits that the FFI uses).
- Independence: I read the code first and formed my own list. Where I agree with an earlier finding I cite its ID. I disagree with one count (FFI-CR-15).

## Recommendation

**APPROVE WITH CHANGES.**

- The code does what the ABI document and the header say. I found no defect that corrupts memory when the caller follows the contract, except the missing length check (FFI-CR-01).
- Changes to make before FFI-001 builds on this file: FFI-CR-01 (in flight), FFI-CR-02, FFI-CR-03, FFI-CR-04, FFI-CR-10. The rest can follow.
- FFI-CR-04 and FFI-CR-05 are design findings. The current structure (raw `Box` pointers, 8 unsafe blocks) does not survive FFI-003. They do not block anything today.
- This is not a security approval. It is not an approval of the in-flight edits.

## Findings

| ID | Severity | Location (a57d899) | Title | Related |
|---|---|---|---|---|
| FFI-CR-01 | high | `lib.rs:198-205` | `len > isize::MAX` reaches `from_raw_parts`, which is undefined behaviour | SEC-F-01 (agree), in flight |
| FFI-CR-02 | medium | `lib.rs:153-164`, `218-233`, `241-254` | Out parameters are not initialised, so a caller that reads them after a failure reads garbage | SEC-F-10 (agree) |
| FFI-CR-03 | medium | `lib.rs:161,179,196,203,225,229,246,251` | SAFETY comments name only non-null. Alignment, liveness and exclusivity rest on the caller. References are created before the other arguments are validated | SEC-F-05 (not re-read) |
| FFI-CR-04 | medium | `lib.rs:36-39,153-182` | The raw-pointer handle design has to change for FFI-003, and the change breaks five signatures | SEC-F-04, ARCH-F-01, ARCH-F-03 |
| FFI-CR-05 | medium | whole file | No safe handle-table shape is written down yet. Proposal below | ADR-0011, ADR-0012 |
| FFI-CR-06 | low | `lib.rs:96-111` | `guard` runs code outside `catch_unwind`. `AssertUnwindSafe` has no justification. The default panic hook writes to the host's stderr | SEC-F-03, ADR-0011 |
| FFI-CR-07 | low | `lib.rs:3-7,115,121,134,173` | Crate docs say every export uses `guard`. Four do not | SEC-F-13, ARCH-F-09 (agree) |
| FFI-CR-08 | low | `lib.rs:74-94,131-146` | Error-text edge cases: stale text if the write silently fails, the "valid until the next call" wording, and thread lifetime | SEC-F-12 (agree) |
| FFI-CR-09 | low | `lib.rs:48-57` | Error mapping uses a wildcard. The code table lives in three places and nothing compares them | ARCH-F-14 (agree) |
| FFI-CR-10 | medium | `lib.rs:257-358` | Test coverage gaps per export | SEC-F-18 (not re-read) |
| FFI-CR-11 | low | `lib.rs:113-117,269-272` | The ABI version test restates the formula. The shift has no range assert | |
| FFI-CR-12 | low | `include/mkpdf.h` | Header text gaps. Signatures match | ARCH-F-03, ARCH-F-05 (agree) |
| FFI-CR-13 | low | `Cargo.toml`, `lib.rs:9-10` | Crate-wide `allow(unsafe_code)` and `allow(missing_safety_doc)`. Crate types. Doc collision | SEC-F-17, ARCH-F-16 (agree) |
| FFI-CR-14 | low | `lib.rs:204,207` | A title may contain NUL bytes. The writer must escape them | PDF-001 |
| FFI-CR-15 | low | whole file | Unsafe block count: I count 8 in non-test code and 5 in tests. One review text says 6 | |

0 critical, 1 high, 4 medium, 10 low. Nothing is resolved by this review.

## Conformance of the exports (checked line by line)

Compared with `include/mkpdf.h` and `docs/api/c-abi.md`.

| Export | Rust (line) | Header | Match |
|---|---|---|---|
| `mkpdf_abi_version` | `() -> u32`, 115 | `uint32_t (void)` | yes |
| `mkpdf_version` | `() -> *const c_char`, 121 | `const char *(void)` | yes |
| `mkpdf_last_error_message` | `() -> *const c_char`, 134 | `const char *(void)` | yes |
| `mkpdf_document_new` | `(*mut *mut MkpdfDocument) -> i32`, 153 | `int32_t (MkpdfDocument **)` | yes |
| `mkpdf_document_free` | `(*mut MkpdfDocument)`, 173 | `void (MkpdfDocument *)` | yes |
| `mkpdf_document_set_title` | `(*mut, *const c_char, usize) -> i32`, 190 | `(MkpdfDocument *, const char *, size_t)` | yes |
| `mkpdf_document_add_page` | `(*mut, f64, f64, *mut usize) -> i32`, 218 | `(MkpdfDocument *, double, double, size_t *)` | yes |
| `mkpdf_document_page_count` | `(*const, *mut usize) -> i32`, 241 | `(const MkpdfDocument *, size_t *)` | yes |

- Status code values at lines 24-34 equal the header macros and the table in `c-abi.md`: 0, 1, 2, 3, 4, 100.
- Nullability matches the documents: `doc` null returns `NULL_POINTER`; `utf8` null with `len == 0` is accepted, with `len > 0` returns `NULL_POINTER`; `out_index` may be null; `out` and `out_count` may not; `free(NULL)` is a no-op.
- Type widths: `size_t`↔`usize`, `int32_t`↔`i32`, `uint32_t`↔`u32`, `double`↔`f64`. No casts and no narrowing occur anywhere in the file, so there is no truncation to review.
- The earlier `nm` check (8 `mkpdf_*` symbols) was run on the working tree, not on `a57d899`. The symbol list in the source has 8 `#[unsafe(no_mangle)]` exports (lines 114, 120, 133, 152, 172, 189, 217, 240).

## Verified correct (read, not just assumed)

- `mkpdf_version` (121-129): the `static` is a `&CStr` built in a const context from `concat!(CARGO_PKG_VERSION, "\0")`. The pointer has `'static` lifetime and never changes. The `Err(_) => c"unknown"` arm cannot run, but it keeps the const evaluation total. The string has no interior NUL. The function cannot panic.
- `set_last_error` and `clear_last_error` use `try_with` and `try_borrow_mut`, so they do not panic during thread teardown. Only the sanitised string reaches `CString::new`, so that call cannot fail on an interior NUL.
- In `mkpdf_document_add_page` the `&mut Document` is last used at line 227, before the write at line 230. No reference is live across the raw write, so the write cannot alias it.
- `Box::into_raw` at 158 and `Box::from_raw` at 180 pair correctly for a unique, live handle.
- The error text is per thread (probe below confirmed isolation).

## What I ran (macOS arm64, rustc/cargo 1.100.0 from Homebrew)

I extracted `a57d899` with `git archive` into the session temp folder and built with a separate `CARGO_TARGET_DIR`. The repository tree was not touched.

| Check | Result |
|---|---|
| `cargo fmt --all --check` on the extract | Passed |
| `cargo clippy --workspace --all-targets -- -D warnings` on the extract | Passed |
| `cargo test -p mkpdf-ffi` on the extract | 6 passed |
| Four extra tests in the temp copy only (not committed): `page_count` with a valid `doc` and null `out_count`; adding `MAX_PAGES + 1` pages; error text isolation between threads; error text after `mkpdf_version` and `mkpdf_document_free(NULL)` | All passed. The limit returned status 4. `out_count` null returned `NULL_POINTER` with text naming `out_count`. A second thread sees no error. The error text is kept after `mkpdf_version` and `free(NULL)` |
| `diff` of `include/mkpdf.h` at `a57d899` against the working tree | Identical |

## What I did not run or examine

- Miri was not run (`rustup` and nightly are not installed). The soundness statements here come from reading, not from a tool.
- No sanitizer, no fuzzing, no Linux or Windows build, no MSRV 1.85 build, and no CI (it has never run).
- I did not re-run the C smoke program against `a57d899`. The earlier smoke test used the working tree at that time and the same header.
- I did not review `mkpdf-core` beyond `model.rs` lines 60-110 and the list of public functions.
- I did not re-review the security analysis of the unsafe code. The security findings cited above are taken from `gate-1-security.md` and not independently re-derived, except where marked "agree".
- The behaviour of the standard library that I rely on (for example, that an unwinding panic out of an `extern "C"` function aborts) is from knowledge, not from a run here.

## Findings in detail

### FFI-CR-01 (high): length is not checked before `from_raw_parts`

- Location: `lib.rs:198-205`. `len` is a `usize`. When `len` is non-zero and `utf8` is non-null, `std::slice::from_raw_parts(utf8.cast::<u8>(), len)` runs. The documented safety rule for that function says the total size must not exceed `isize::MAX`.
- A binding that converts a negative signed length into `size_t` produces `len > isize::MAX`. That is undefined behaviour in Rust before any byte is read, and the null checks do not catch it. I rate it high because a plain conversion mistake in a wrapper reaches it, with no hostile caller needed.
- Proposed fix: before the slice, `if len > isize::MAX as usize { return Err(Failure::InvalidArgument("len exceeds isize::MAX")) }` (needs a `Failure` variant that maps to `MKPDF_ERR_INVALID_ARGUMENT`, or use `LIMIT_EXCEEDED`). Add a test with `len = usize::MAX` and a non-null pointer to a small buffer, which must return the error and not read. Also set the string limit that the core gains (SEC-F-02), so that the check and the limit are one rule. State both in `c-abi.md`.
- Status: SEC-F-01 is already being fixed by the Interoperability Engineer. I agree with that finding.

### FFI-CR-02 (medium): out parameters are not initialised

- Location: `mkpdf_document_new` (153-164) writes `*out` only on success. `mkpdf_document_add_page` writes `*out_index` only on success (228-231). `mkpdf_document_page_count` writes `*out_count` only on success (248-252).
- Effect: a caller that does not check the status, or a wrapper that reads the out value on a failure path, reads whatever the variable held. For `new` this is a garbage pointer that the caller may later pass to `free`.
- Proposed fix: write the neutral value as soon as the pointer is known non-null and before anything can fail. For `new`: after the null check, `*out = ptr::null_mut()`. For `page_count`: write `0` to `*out_count` first. For `add_page`: there is no neutral index (0 is a valid page index), so either document `*out_index` as "not written on failure" or write `usize::MAX` first. I propose the second, with `usize::MAX` documented as "no page", because it is checkable by a caller. The owner decides, since it is ABI behaviour. `new` always writes `NULL` first. Record the rule in `c-abi.md` and in the header. Add one test per function that pre-fills the out value with a sentinel and checks it after each failing call.
- Note for FFI-003: with a token handle, the out value becomes `0`, which is the invalid token.

### FFI-CR-03 (medium): what the SAFETY comments actually guarantee

For each unsafe block, whether the invariant it names is guaranteed by the code or only by the caller:

| Line | Block | Comment says | Guaranteed by | Can it be removed or narrowed? |
|---|---|---|---|---|
| 162 | `*out = handle` | non-null and writable | non-null by code (155). Writable and aligned by caller | No. Could use `out.write(handle)`. After FFI-CR-02 it runs twice, so combine into one helper |
| 180 | `Box::from_raw(doc)` | came from `Box::into_raw`, unique | Caller only. Null is checked. Nothing else is | Removed by FFI-003. No unsafe is needed once handles are table tokens |
| 197 | `doc.as_mut()` | non-null; caller guarantees liveness and exclusivity | Non-null by code. Alignment, liveness, exclusivity: caller only | Narrow it. See below |
| 204 | `from_raw_parts` | non-null and `len` readable bytes | Non-null by code. Readable by caller. `len <= isize::MAX` by nobody (FFI-CR-01) | Cannot be removed. Add the length check above it |
| 226 | `doc.as_mut()` | "see set_title" | as 197 | as 197 |
| 230 | `*out_index = index` | non-null and writable | non-null by code. Writable by caller | Could use `.write()`. Otherwise needed |
| 247 | `doc.as_ref()` | non-null; caller guarantees liveness | as 197, shared reference | as 197 |
| 252 | `*out_count = ...` | non-null and writable | non-null by code | needed |

- `<*mut T>::as_mut` is safe to call only if the pointer is aligned, dereferenceable, points to an initialised value, and no other reference or access conflicts for the reference's lifetime. The comments at 196 and 246 say "non-null checked by `as_mut`", which is only the first of those. The comment should state all four and say plainly that the last three come from the caller contract.
- Order: in `set_title` the `&mut Document` is created (197) before the `utf8` null check and the UTF-8 check. In `add_page` the `&mut` is created before `Page::new` runs. So the exclusive reference lives across code that can fail and can panic. If any of the other arguments is bad the reference was still made. Creating the reference last, after every argument is validated, shortens its lifetime and costs nothing.
- Proposed fix: one private helper, for example `fn with_doc<R>(doc: *mut MkpdfDocument, f: impl FnOnce(&mut Document) -> Result<R, Failure>) -> Result<R, Failure>`. It holds the only `as_mut` in the file, confines the borrow to the closure, and has one SAFETY comment that states all four conditions. A second helper `write_out<T>(ptr: *mut T, value: T)` holds the writes. After this, `set_title`, `add_page` and `page_count` need no `unsafe` of their own except the slice. When FFI-003 lands, only the body of `with_doc` changes (FFI-CR-05).
- Enable `clippy::undocumented_unsafe_blocks` so that a missing comment is a build error (SEC-F-17).

### FFI-CR-04 (medium): what in the current structure must change for FFI-003

- `MkpdfDocument` is a real struct behind `Box::into_raw` (36-39, 158). A pointer cannot be validated without reading it, and reading freed memory is undefined behaviour. So no check of any kind can be added to the existing design. The handle has to become a token that is never dereferenced. This agrees with SEC-F-04 and ARCH-F-01.
- Signatures that change: `mkpdf_document_new` (out is a token, not a pointer), `mkpdf_document_free` (must return a status to report a stale handle, today it returns `void`), `set_title`, `add_page` and `page_count` (first parameter). The header changes with them. This is an ABI break, so the version rule in ARCH-F-03 has to be settled first.
- Unchanged: the error-text mechanism, `guard`, the status codes (one new code is needed), `mkpdf-core`.
- A new status code is needed for a stale or foreign handle, and probably a second for "handle in use by another thread" (see FFI-CR-05). The values are an owner decision.
- Header typedef: I recommend a plain `uint64_t` token, for example `typedef uint64_t MkpdfDocument;`. Reason: a pointer-typed token that is not a real pointer is hostile to runtimes that inspect pointers (Go's cgo pointer checks, debuggers, address sanitizers) and invites code that dereferences it. A 64-bit integer is opaque to every wrapper (ADR-0012 treats buffers the same way).

### FFI-CR-05 (medium): the smallest safe shape for the handle table

This is a proposal for the FFI-003 ADR, which is not written. It uses no dependency.

```rust
struct Slot { generation: u32, doc: Option<Arc<Mutex<Document>>> }
struct Table { slots: Vec<Slot>, free: Vec<u32>, live: usize }
static TABLE: Mutex<Table> = Mutex::new(Table { slots: Vec::new(), free: Vec::new(), live: 0 });
const MAX_LIVE_HANDLES: usize = 1 << 20; // value is a decision, covered by a boundary test
```

- Token: `(u64::from(generation) << 32) | u64::from(index)`. The first generation is 1, so the token `0` is never valid. Anything not in the table, including a forged value, is rejected by the index and generation check.
- `new`: lock the table, take a free index or push a slot, store `Arc::new(Mutex::new(Document::new()))`, return the token. Refuse with `LIMIT_EXCEEDED` past `MAX_LIVE_HANDLES`.
- Lookup: lock the table, check `index < slots.len()`, `generation` equal, `doc` is `Some`, clone the `Arc`, release the table lock. Then `try_lock` the document mutex. `WouldBlock` returns a "busy" status. That enforces the one-thread-at-a-time contract that today is only written down. `Poisoned` returns `MKPDF_ERR_INTERNAL`. A panic inside the closure drops the guard during unwinding, and the std mutex marks itself poisoned, which is exactly the "poisoned flag" in ADR-0011 item 4 at no extra cost.
- `free`: lock the table, check the token, take the `Arc`, bump `generation` (retire the slot if the counter would wrap, rather than reuse it), push the index on the free list, release the lock, then drop the `Arc`. A call that is still running on another thread keeps its clone, so the document is dropped when that call ends. Free of a stale or already freed token returns the new status and never touches memory. Free must work on a poisoned document so a caller can always clean up.
- Lock order: never hold the table lock and a document lock together, so there is no deadlock path. The table lock covers only index arithmetic that cannot panic. Recover from a poisoned table mutex with `unwrap_or_else(PoisonError::into_inner)` (this is allowed by the workspace lints, which deny only `unwrap`/`expect`).
- Unsafe left: the three out-parameter writes and the one `from_raw_parts` (4 blocks instead of 8). `free` needs no unsafe at all.
- ADR-0012: the output buffer is a second table of the same kind with `Arc<Vec<u8>>`. Write the table for documents first and generalise it when the buffer exists, so the abstraction has two uses (`rules/architecture.md`, no abstraction without a second use).
- Costs and open points for the ADR: a global lock on every call (acceptable now; a read-write lock or sharding later), a new `static` (the "no global state" rule in `rules/architecture.md` item 2 is stated for `mkpdf-core`, and this lives in the FFI crate, but the ADR should say so), and a slot is a few dozen bytes per live handle. The table is never built for `wasm32`, since the FFI crate is not built there.
- Wrapper effect: every wrapper holds one `uint64`. A stale handle becomes an error, not a crash. Wrappers should still set the token to 0 after freeing.

### FFI-CR-06 (low): the shape of `guard`

- `catch_unwind` wraps only `f()` (97). The follow-up work runs outside it: `clear_last_error`, `failure.message()` (a `format!` and a `to_owned`) and `set_last_error` (a `replace` and a `CString::new`). A panic there would cross the `extern "C"` boundary. Today none of these can panic (allocation failure aborts, it does not unwind), so this is not a defect. It is a structural gap that ADR-0011 should close, for example by running the whole body, including error recording, inside one `catch_unwind`, with a fixed-size fallback message.
- `AssertUnwindSafe` (97) has no comment saying why the closure is unwind-safe. The honest reason: today every mutation is one assignment or a `push` after the limit check, so a panic cannot leave the document half changed. That reason stops holding when the writer arrives. Put the reason in a comment and tie it to ADR-0011 item 4.
- The Rust default panic hook prints the panic message and source path to stderr of the host process. A library that is loaded into a server therefore writes to its host's stderr on every contained panic. Installing a hook from a library is also hostile to the host. This belongs in ADR-0011 as an open point; I have not changed that ADR.
- `catch_unwind` payload (`Err(_)`) is dropped inside `guard`; a payload whose destructor panics would abort. Not reachable from this code.
- The `panic_is_contained` test (353-357) checks `guard` only, as ARCH-F-04 says. No test panics through an exported function.

### FFI-CR-07 (low): crate documentation overstates `guard` use

- Lines 3-7 say every exported function runs inside `guard`. `mkpdf_abi_version` (115) and `mkpdf_version` (121) are plain functions, `mkpdf_last_error_message` (134) and `mkpdf_document_free` (173) call `catch_unwind` directly. None can panic as written, so I agree with SEC-F-13 and ARCH-F-09 that the behaviour is reasonable and the text is wrong. `c-abi.md` was already corrected.
- Proposed fix: change the crate doc to say what the code does, or give `mkpdf_document_free` a status-returning form under FFI-003, which makes it go through `guard`. The rule text in `rules/rust-engineering.md` item 5 is the owner's to amend.
- The doc comment at line 4 also links to the private `guard` (the `cargo doc` warning in ARCH-F-16).

### FFI-CR-08 (low): error-text edge cases

- If `try_with` or `try_borrow_mut` fails inside `set_last_error` (78-86), the function does nothing and the previous text stays. The status is an error but the text is stale or `NULL`. This is not reachable today (no re-entrancy, and teardown only). A fixed fallback could not be set either, because the slot is the thing that failed. Keep it, and say in a comment that it is unreachable and why.
- The wording "valid until the next mkpdf call on the same thread" (header, `c-abi.md`, line 132) is stricter than the code. Only guarded calls replace the text, as probed. That is the safe direction. No change needed, but a binding must not rely on the text surviving other calls.
- The pointer is into a thread-local `CString`. It dies at the next guarded call and at thread exit. For runtimes that migrate calls between OS threads, the text can be read on the wrong thread (SEC-F-12, agree). The fix is documentation and wrapper rules, as in ADR-0012.

### FFI-CR-09 (low): error mapping

- `_ => MKPDF_ERR_INVALID_ARGUMENT` (54) is required across crates for a `#[non_exhaustive]` enum. A new core kind silently degrades. I agree with ARCH-F-14.
- The status values exist in three places (Rust constants, header macros, `c-abi.md`) and nothing compares them. Proposed fix: one test that includes the header text (`include_str!("../../../include/mkpdf.h")`) and checks every `#define MKPDF_ERR_*` and `MKPDF_OK` value against the Rust constants. This is cheap and works before cbindgen is in CI. A C compile test in CI remains a separate need (ARCH-F-05).

### FFI-CR-10 (medium): test coverage per export

At `a57d899` there are 6 tests (the test count was confirmed by a run). What each export has and lacks:

| Export | Covered | Missing |
|---|---|---|
| `mkpdf_abi_version` | `abi_version_packs_major_minor` | The test restates the formula, so it cannot catch a wrong constant (FFI-CR-11) |
| `mkpdf_version` | Content equals `CARGO_PKG_VERSION` | Nothing important |
| `mkpdf_last_error_message` | Set after a failure, `NULL` after a success, via the helper | Thread isolation, the text surviving `mkpdf_version` and `free`, and `NULL` before any call. All three pass in my temp probe, none is committed |
| `mkpdf_document_new` | Success. Null `out` | Out value on failure (FFI-CR-02). A panic path |
| `mkpdf_document_free` | Valid and `NULL` | Nothing testable beyond that (a double free is undefined by contract). Miri needs to run it |
| `mkpdf_document_set_title` | Success, bad UTF-8, `NULL` with `len` 3, `NULL` with `len` 0 | Null `doc`. `len > isize::MAX` (FFI-CR-01). Setting twice. A title with a NUL (FFI-CR-14) |
| `mkpdf_document_add_page` | Success with and without `out_index`. NaN. Null `doc` | Zero, negative, infinite and over-limit sizes through the FFI. `MAX_PAGES + 1` (status 4 in my probe, but no committed test). `out_index` untouched on failure |
| `mkpdf_document_page_count` | Success. Null `doc` together with null `out_count` | Valid `doc` with null `out_count` (passes in my probe). The existing test passes both null, so it cannot tell which check fired |
| `guard` | A caught panic | A panic through an exported function. `Ok(Err)` leaving the old text replaced |

Other gaps: no C compile-and-link test in the repository, no Miri run, no check that the exported symbol set equals the header (I did that by hand earlier). The tests use `.unwrap()` and `unsafe` freely, which `clippy.toml` allows in tests.

### FFI-CR-11 (low): the ABI version

- `mkpdf_abi_version` (115) computes `(ABI_MAJOR << 16) | ABI_MINOR`. If `ABI_MINOR` ever reaches 65536 it would corrupt the major field without any error. Add `const _: () = assert!(ABI_MINOR < 1 << 16 && ABI_MAJOR < 1 << 16);`.
- The test (269-272) uses the same expression as the code. Test the literal: `assert_eq!(mkpdf_abi_version(), 0x0000_0001)`, and compare to the header values as in FFI-CR-09.

### FFI-CR-12 (low): header

- All signatures and values match (table above). Gaps in the text: the header does not say what happens to out parameters on failure (FFI-CR-02), does not mention the thread rule, and does not say that `len` is limited. `mkpdf_abi_version` says "Refuse to run if the major version differs", which is the 0.x ambiguity in ARCH-F-03. The macro names `MKPDF_ABI_MAJOR` differ from the Rust constant names `ABI_MAJOR`, so cbindgen would not reproduce the header (ARCH-F-05, agree).

### FFI-CR-13 (low): `Cargo.toml` and lints

- `[lib] name = "mkpdf"` with `crate-type = ["cdylib", "staticlib", "rlib"]` is intentional for `libmkpdf`. It collides with the CLI binary in `cargo doc` (ARCH-F-16). The `rlib` is what the unit tests link against.
- No `panic` key in the profiles (ARCH-F-04, ADR-0011).
- `#![allow(unsafe_code)]` at line 9 applies to the whole file, including code that has no unsafe. After FFI-CR-03 the unsafe is in two small helpers, so the allow could be placed on those items and on the exports. I have not checked whether the `#[unsafe(no_mangle)]` attribute alone trips the `unsafe_code` lint, which would force the allow onto every export. Check that before moving it.
- `#![allow(clippy::missing_safety_doc)]` at line 10 is not needed: every `unsafe extern "C" fn` has a `# Safety` section (SEC-F-17, agree).

### FFI-CR-14 (low): NUL bytes in a title

- Valid UTF-8 may contain U+0000. `set_title` accepts it (207), so the document can hold a title with a NUL. It is harmless here, and the PDF writer must escape or reject it. Add a test now and a rule in the PDF-001 acceptance criteria.

### FFI-CR-15 (low): unsafe block counts

By my count of the committed file:

- Non-test `unsafe { ... }` blocks: **8**, at lines 162, 180, 197, 204, 226, 230, 247, 252.
- Test `unsafe { ... }` blocks: **5**, at lines 266, 277, 284, 313, 330.
- Also: 5 `unsafe extern "C" fn` declarations (153, 173, 190, 218, 241) and 8 `#[unsafe(no_mangle)]` attributes. These are not blocks, and the three counts are easy to confuse.

This matches your count. `docs/reviews/gate-1-security.md:20` says "6 in non-test code", but the list that follows it names 8 blocks (new write; free `from_raw`; `set_title` `as_mut` and `from_raw_parts`; `add_page` `as_mut` and write; `page_count` `as_ref` and write). So the number "6" in that sentence is wrong and the enumeration is right. I did not edit that file. Several tests put more than one unsafe call in a single block (284-308, 313-324, 330-349), which `clippy::multiple_unsafe_ops_per_block` would flag.

## Statement on approval

This review approves nothing under `rules/security.md`. It does not accept any risk. The Security Engineer must review the in-flight changes to `mkpdf-ffi`, and the owner decides the ABI items (handle token type, new status codes, version rule).

## Next review

Re-review the working tree after the SEC-F-01 fix and the core title limit land, and again when the FFI-003 ADR is written.
