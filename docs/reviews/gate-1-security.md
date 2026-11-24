# GATE-1 security review

- Reviewer: Security Engineer (agent role, claude-sonnet-5-5). This is a single-reviewer review, not an independent second review.
- Date: 2026-11-23
- Repository state: commit `59df15e` (foundation, milestone 0), working tree as found. The only edits made by this review are `docs/security/threat-model.md` and this file.
- Related, not reviewed here: ADR-0011 (panic strategy) and ADR-0012 (output-buffer ownership) are being drafted by another agent. `docs/reviews/gate-1-architecture.md` belongs to another agent.

## Recommendation

**PASS WITH CONDITIONS** for closing the security part of GATE-1 (foundation milestone).

- No critical finding and no high finding was found. Nothing here blocks the foundation commit.
- This is **not** an approval for a production release. 19 findings are open (0 critical, 0 high, 8 medium, 11 low). None is resolved, and no evidence of a fix exists yet. A production release needs a new review.
- Conditions (C-1 to C-6) are in the section "Conditions" below. C-1 to C-3 must be met before FFI-001 (the first function that takes a caller buffer or returns one) merges.
- Medium and low findings have a proposed owner and date below. Those are proposals. The Orchestrator must confirm them. Only the project owner can accept a risk, in writing in the repository. No risk acceptance exists today. SEC-F-04 and SEC-F-09 need an owner decision.

## What was examined

- Read in full: `AGENTS.md`, all seven files in `rules/`, `docs/security/threat-model.md`, `docs/requirements.md`, ADR-0003, ADR-0005, `docs/api/c-abi.md`, `include/mkpdf.h`, `crates/mkpdf-ffi/src/lib.rs`, `crates/mkpdf-core/src/{lib,model,error}.rs`, `crates/mkpdf-cli/src/main.rs`, root `Cargo.toml`, the crate manifests, `deny.toml`, `.github/workflows/ci.yml` (the only workflow file), `SECURITY.md`, `rust-toolchain.toml`, `clippy.toml`, `.gitignore`.
- Manual review of every `unsafe` block (6 in non-test code: `mkpdf_document_new` write, `mkpdf_document_free` `Box::from_raw`, `set_title` `as_mut` and `from_raw_parts`, `add_page` `as_mut` and write, `page_count` `as_ref` and write) and of all 8 exported functions.
- Checked the exported symbol list of the release `cdylib` with `nm -gU target/release/libmkpdf.dylib`: exactly 8 `mkpdf_*` symbols, and their names match the header.
- Searched the release `cdylib` for the build user's home path: no match (macOS arm64 only).

## What was run

| Check | Result (2026-11-23, macOS arm64, local) |
|---|---|
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed, no issues |
| `cargo test --workspace --locked` | Passed: core 4 tests, ffi 6 tests, others 0 |
| `cargo build -p mkpdf-ffi --release --locked` | Passed |
| `cargo deny check` (cargo-deny 0.21.0) | Passed: advisories ok, bans ok, licenses ok, sources ok. Four `license-not-encountered` warnings (MIT, ISC, Unicode-3.0, Zlib). The workspace has no third-party crate, so this run checked almost nothing |

## What was NOT run or not examined

- **`cargo audit`: not run.** It is not installed. I did not install it. `cargo deny` checks the same advisory database, but the rules and CI call for `cargo audit` as well.
- **Miri: not run.** `rustup` and `cargo-miri` are not available here and the local toolchain is a Homebrew stable `rustc`. No test of the FFI crate has run under Miri, so aliasing and provenance claims in this review come from reading, not from a tool.
- **CI has never run.** Nothing in `ci.yml` is verified, including the Miri, MSRV, wasm32, Linux aarch64 and Windows jobs.
- No fuzzing, no sanitizers (ASan, TSan), no C or C++ caller test. No such harness exists.
- Not examined: Linux and Windows builds, library unload behaviour (`dlclose`, `FreeLibrary`), symbol export control of the `staticlib`, behaviour with `panic = "abort"` (not built), the commit history beyond one foundation commit, `docs/dependencies.md`, ADRs 0001, 0002, 0004, 0006 to 0010 (only checked where they bear on the threat model), `docs/architecture.md`, `docs/roadmap.md`, `docs/testing.md`, `CONTRIBUTING.md`, `.opencode/agents/`.
- Not examined: GitHub repository settings (branch protection, required reviews, secret scanning, Dependabot, private vulnerability reporting). They cannot be seen from the checkout.
- `mkpdf-core` was read in full. It is 4 small files, `forbid(unsafe_code)`, no dependencies, and no input parsing exists yet. There is no parser, serialiser, crypto or network code to review.

## Findings

Severity scale: critical (memory corruption or code execution reachable by hostile input, with no contract violation needed), high (crash or UB reachable through plausible misuse, or a blocking control missing), medium (defence in depth missing, or a gate control unverified), low (hardening, documentation or process).

| ID | Severity | Title | Status |
|---|---|---|---|
| SEC-F-01 | medium | `slice::from_raw_parts` receives an unchecked `len` that can exceed `isize::MAX` | Fixed in the working tree, verified 2026-11-23, **not committed**. Becomes Resolved when committed with the tests unchanged (see "Re-review: set_title fix") |
| SEC-F-02 | medium | No size limit on the title string; allocation failure aborts the host | Partly fixed in the working tree, **not committed**. Code and tests done. The limit is not in the C ABI doc or the header (see "Re-review: set_title fix", SEC-F-20) |
| SEC-F-03 | low | Handle state after a caught panic is not defined (sound today, latent) | Open |
| SEC-F-04 | medium | Double free and use after free are undefined behaviour with no detection | Open, owner decision |
| SEC-F-05 | medium | Miri has not been run on the FFI crate | Open |
| SEC-F-06 | medium | CI toolchain steps are probably overridden by `rust-toolchain.toml` (Miri and MSRV jobs) | Open, unverified |
| SEC-F-07 | medium | CI actions are not pinned to commit SHAs; `cargo install cargo-audit` is unpinned | Open |
| SEC-F-08 | low | `deny.toml` gaps | Open |
| SEC-F-09 | medium | No private vulnerability reporting channel | Open, owner action |
| SEC-F-10 | low | Out parameters are not initialised on failure; alignment unchecked | Open |
| SEC-F-11 | low | Aliasing and thread contract is not enforced; no reentrancy rule | Open |
| SEC-F-12 | low | Thread-local error text has thread-migration and unload caveats | Open |
| SEC-F-13 | low | Three exported functions bypass `guard`, against the rule and the ABI doc | Open |
| SEC-F-14 | medium | Panic strategy is undecided and `panic = "unwind"` is not pinned or tested | Open, ADR-0011 pending |
| SEC-F-15 | low | Hand-written header can drift from the implementation | Open |
| SEC-F-16 | low | Toolchain and CI tool versions float | Open |
| SEC-F-17 | low | Blanket lint allows in the FFI crate; `SAFETY:` comments not enforced by lint | Open |
| SEC-F-18 | low | Release artefacts may embed build paths (Linux and Windows not checked) | Open |
| SEC-F-19 | low | Test gaps at the boundary | Partly addressed, **stays Open** (re-checked 2026-11-24 at `ffa10a6`). Added since GATE-1, committed in `eeed518` and `4a0751b`: `len = usize::MAX` and string-limit tests, `set_title_null_utf8_with_huge_len_is_null_pointer`, `set_title_accepts_interior_nul`, `max_pages_is_enforced_through_ffi`. Still missing: a panic through a real exported function, a thread test for the error text, `mkpdf_version` and `MKPDF_MAX_TITLE_BYTES` checked against the header, non-null `utf8` with `len = 0`, the C compile test, and a `tests/` directory. The new tests have not run in CI or Miri (`4a0751b` not pushed). See `docs/reviews/security-adr-0011-0013.md`, "Re-review after amendments" |
| SEC-F-20 | low | Title limit and new `set_title` signature are not in the C ABI doc, header, Rust API doc, threat model or changelog | **Mostly fixed in `4a0751b`, stays Open for two small items** (re-checked 2026-11-24). Done: `docs/api/c-abi.md:67-82`, `include/mkpdf.h:25-26` and `45-57`, `docs/api/rust-api.md:13` and `:30`, `CHANGELOG.md` "Changed" and "Fixed". Threat model T-04 and T-19 corrected by the Security Engineer on 2026-11-24. Open: `model.rs:10-11` still points to T-10 instead of T-04, and no test checks the header constant against `MAX_TITLE_BYTES` (rust-core, qa; due 2026-12-07) |
| SEC-F-21 | low | `set_title` SAFETY comment and `# Safety` text omit two `from_raw_parts` conditions; one test comment overclaims | **Resolved in `4a0751b`** (verified 2026-11-24 at `ffa10a6`). Evidence: `# Safety` text at `lib.rs:209-216` and the SAFETY comment at `lib.rs:236-241` state initialised bytes, one allocation and no change during the call; check order documented at `lib.rs:198-205`; `const _: () = assert!(MAX_TITLE_BYTES <= isize::MAX as usize)` at `lib.rs:37`; the test comment at `lib.rs:403-406` now says release `cargo test` does not prove the order; new test `set_title_null_utf8_with_huge_len_is_null_pointer` checks the documented precedence. `cargo test` passes (15 FFI tests). The remaining test gaps are tracked under SEC-F-19 |

### SEC-F-01 (medium): unchecked `len` in `slice::from_raw_parts`

- Evidence: `crates/mkpdf-ffi/src/lib.rs:198-205`. When `len != 0` and `utf8` is non-null, `std::slice::from_raw_parts(utf8.cast::<u8>(), len)` is called with any `usize`.
- Problem: `from_raw_parts` requires `len` bytes to be one allocation and the total size to be at most `isize::MAX`. A larger `len` is undefined behaviour even if the bytes are never read. In practice this comes from SDK bugs: a negative `int` or `long` converted to `size_t` gives a value near `usize::MAX` (JNI, P/Invoke and CGo wrappers all do such conversions). Standard-library debug assertions would abort a debug build, a release build continues into UB or a segfault. The documented contract ("readable for `len` bytes") technically excludes this, but we cannot rely on every wrapper being right, and the check costs one comparison.
- Proposed fix: before building the slice, return `MKPDF_ERR_INVALID_ARGUMENT` (or `LIMIT_EXCEEDED`) when `len > isize::MAX as usize`. Better, reject anything above a documented maximum string size (SEC-F-02), which implies the first check. Add a test with `len = usize::MAX` and a non-null pointer to a small buffer. The test must not dereference, so it only passes if the check comes first.
- Disposition: fix before FFI-001. Owner: rust-core. Proposed date: 2026-12-07. Reviewer: security.

### SEC-F-02 (medium): no title size limit; OOM aborts

- Evidence: `lib.rs:206-207` and `crates/mkpdf-core/src/model.rs:74-76`. `set_title` copies the string without a limit. `MAX_PAGES` and `MAX_PAGE_DIMENSION_PT` exist (`model.rs:5-8`), but nothing bounds strings. `rules/architecture.md` item 6 requires a documented limit for every input-driven allocation.
- Problem: a caller can force a copy of up to `isize::MAX` bytes. If allocation fails, Rust aborts the process, and `catch_unwind` cannot stop that. Page count is bounded (100 000 pages of 16 bytes each, small), so pages are not a concern at present.
- Proposed fix: add `MAX_STRING_BYTES` (for example 64 KiB for metadata) in `mkpdf-core`, returned as `LimitExceeded`, covered by a boundary test, and document it in `docs/api/c-abi.md`. State in the ABI doc that allocation failure aborts.
- Disposition: fix before v0.1.0 and before any new string input. Owner: rust-core. Proposed date: 2026-12-07.

### SEC-F-03 (low): handle state after a caught panic

- Evidence: `lib.rs:96-111` (`guard` uses `AssertUnwindSafe`), `lib.rs:197-208`, `lib.rs:226-232`.
- Analysis: `AssertUnwindSafe` is the correct tool here and is not unsound by itself. The closures mutate the document through a raw pointer, so a panic mid-update could leave it half-modified. Reading the core code, every mutation today is a single assignment (`self.title = Some(..)`) or a `Vec::push` after the limit check, and none of them can panic (allocation failure aborts instead). So the handle is consistent after a caught panic in the current code. Page validation (`Page::new`) happens before the mutation.
- Risk: this holds by inspection only. Once the writer and content streams exist, a panic between two mutations could leave an invalid document, and the next call would operate on it. Also, `mkpdf_document_free` after a caught panic is fine (the allocation is intact).
- Proposed fix: ADR-0011 should state the rule. Either (a) all mutating core functions validate first and mutate last, enforced by review, or (b) the handle gets a `poisoned` flag set by `guard` and every later call except `free` returns `MKPDF_ERR_INTERNAL`. Option (b) is cheap and robust. Add a test that forces a panic through a real exported function (needs a test-only hook).
- Disposition: decide in ADR-0011, implement before the writer lands. Owner: architect (decision), rust-core (code). Proposed date: 2026-12-14.

### SEC-F-04 (medium): double free and use after free

- Evidence: `lib.rs:167-182`, `docs/api/c-abi.md:17`, ADR-0003 consequences, threat model T-03.
- Problem: `Box::from_raw(doc)` on a freed or foreign pointer is undefined behaviour, and the same is true for any later call with a stale handle (`as_mut` forms a reference to freed memory). SDKs written in garbage-collected languages (finalizer plus explicit `Dispose`) make double free a likely bug, not a remote one. In an in-process library the result is memory corruption in the host. The risk is by design (owner decision D-02) and is the normal C convention, so I rate it medium, not high. If the owner wants to accept it, the acceptance must be written in the repository.
- Note on the option listed in ADR-0003: a magic tag inside the struct does not detect use after free, because reading freed memory is already UB, and the allocator may have reused it. It only catches a foreign pointer that happens to point at readable memory. A handle table (handles are indices with a generation counter, validated under a lock or with atomics) does detect stale handles. It costs a global lock and changes the ABI from pointers to integers or opaque tokens, so the decision belongs before ABI 1.0 and before SDK work in v0.2.0.
- Proposed disposition: owner decides before v0.2.0, with the architect's cost analysis. Options: (A) accept the C contract and require every SDK wrapper to make `free` idempotent in its own language (cheap, covers the likely case), plus sanitizer tests in the binding suites; (B) checked handles. My recommendation is A for v0.1.0 and a recorded decision for B at v0.2.0 planning. Owner: project owner (decision), architect, interop.

### SEC-F-05 (medium): Miri not run

- Evidence: no Miri result exists. `rules/security.md` item 3 and `rules/testing.md` item 7 require it. `rustup` is absent locally. The CI job (`ci.yml:68-75`) has never run (see SEC-F-06).
- Risk: the review of aliasing, provenance and `&raw mut` patterns is by reading only. I found no problem by reading: references are short-lived, created inside `guard`, and written pointers are used once. Note the test at `lib.rs:305` dereferences `(*doc).inner` after other calls, which is fine.
- Proposed fix: run Miri once in CI (or on a machine with nightly) with `MIRIFLAGS=-Zmiri-strict-provenance`, record the version and result in this folder, and make the job required. Do not mark this resolved without the run log.
- Disposition: close with a recorded green Miri run. Owner: qa and devops. Proposed date: 2026-12-07.

### SEC-F-06 (medium): `rust-toolchain.toml` probably overrides the toolchain chosen in CI

- Evidence: `rust-toolchain.toml` pins `channel = "stable"`. Jobs `msrv` (`ci.yml:42`, `dtolnay/rust-toolchain@1.85`) and `miri-ffi` (`ci.yml:72`, `@nightly`) select a different toolchain by setting the rustup default. A `rust-toolchain.toml` in the repository takes precedence over the rustup default.
- Expected effect (not verified, CI has never run): the MSRV job compiles with stable, so it verifies nothing about 1.85, and `cargo miri` runs on stable, where the Miri component is missing, so the job fails or, worse, is skipped by someone fixing it the wrong way. The OQ-04 note already says the MSRV job is unverified.
- Proposed fix: use explicit `cargo +1.85 check` and `cargo +nightly miri test`, or set `RUSTUP_TOOLCHAIN` in the job, or delete the file in those jobs. Verify on a real run that `rustc --version` printed in the log matches the intent. Add a `rustc --version` step to every job.
- Disposition: devops. Proposed date: 2026-12-07.

### SEC-F-07 (medium): CI supply chain

- Evidence, `.github/workflows/ci.yml`:
  - `actions/checkout@v4` (lines 19, 33, 41, 48, 57, 63, 71): mutable tag.
  - `dtolnay/rust-toolchain@stable`, `@nightly`, `@1.85` (lines 20, 34, 42, 49, 64, 72): these are branch names that move. This is a third-party action that downloads and runs a toolchain installer.
  - `EmbarkStudios/cargo-deny-action@v2` (line 58): mutable tag, third-party.
  - `cargo install cargo-audit --locked` (line 65): installs the latest release from crates.io, unpinned, then runs it. `--locked` pins its dependencies, not the tool.
- Good: top-level `permissions: contents: read` (line 8) is least privilege. No secrets are used. The trigger is `pull_request`, not `pull_request_target`, so fork PRs get a read-only token and no secrets. No caching, so no cache poisoning path.
- Missing: `persist-credentials: false` on checkout (the token stays in `.git/config` for later steps; low impact with read-only scope), `timeout-minutes`, `concurrency` cancellation, a Dependabot or Renovate configuration to update pinned SHAs, and CODEOWNERS for `.github/` and `deny.toml` (the rule says the security engineer approves changes to the FFI crate, and nothing enforces it).
- Proposed fix: pin every `uses:` to a full commit SHA with a version comment (`# v4.2.2`), pin `cargo-audit` with `--version`, add Dependabot for `github-actions` and `cargo`, add CODEOWNERS and branch protection with required status checks and required security review for `crates/mkpdf-ffi/**`, `.github/**`, `deny.toml`. Pinning is required before the first release (threat T-13 says so already).
- Disposition: devops. Proposed date: before the first release, and no later than 2026-12-14.

### SEC-F-08 (low): `deny.toml`

- Evidence: `deny.toml:1-30`.
- Good: `yanked = "deny"`, `wildcards = "deny"`, unknown registry and unknown git denied, only crates.io allowed, permissive licence list matches `rules/licensing.md` item 7 exactly.
- Gaps: (1) `multiple-versions = "warn"` is fine now, but the review should decide whether it stays a warning once dependencies exist. (2) The `[bans]` section has no `deny` list. The core must never open network connections (`rules/security.md` 4), so ban network and TLS crates (`openssl`, `openssl-sys`, `native-tls`, `reqwest`, `hyper`, `ureq`) for the core and FFI crates via `wrappers` or per-crate bans, and ban `build.rs`-heavy crates by review. (3) Advisory handling: `unmaintained` and `unsound` default behaviour in the v2 format depends on the cargo-deny version, so set `unmaintained = "workspace"` (or `"all"`) and `unsound = "all"` explicitly. (4) `[licenses]` has four `license-not-encountered` warnings. They are harmless until dependencies exist, but CI should keep the warnings visible so that nobody adds a licence without a dependency that needs it. (5) `wildcards = "deny"` also rejects path dependencies without version. The workspace dependency sets `version`, so it passes today. Set `allow-wildcard-paths` only if needed. (6) No `[graph] targets` list, so the check covers all targets including ones nobody ships. That is conservative and fine.
- Disposition: add the explicit settings when the first dependency is proposed (ADR-0005 process). Owner: security with devops. Proposed date: 2026-12-14.

### SEC-F-09 (medium): no disclosure channel

- Evidence: `SECURITY.md` says "Reporting channel: not yet established". `docs/requirements.md` OQ-01 shows it as an owner action. Threat T-25.
- Risk: today there is nobody to report a vulnerability to. For a repository that is not public yet this is not exploitable, but it becomes high on the day the repository goes public or a release is made. The severity policy blocks a production release while it is open.
- Disposition: project owner enables GitHub private vulnerability reporting and sets the role alias, then updates `SECURITY.md` and `CODE_OF_CONDUCT.md`. Must be done before the repository is made public. Owner: project owner.

### SEC-F-10 (low): out parameters and alignment

- Evidence: `lib.rs:153-165` (`*out` is written only on success), `lib.rs:218-234`, `lib.rs:241-255`.
- Problem: on failure `*out` is left as the caller had it. A caller that ignores the status and then uses an uninitialised pointer variable gets UB in its own code. Writing `NULL` first removes that class of bug and is idiomatic. Also, `*out = handle` and `*out_index = index` are aligned writes, and a misaligned pointer from a caller (for example a packed struct field) is UB. That is covered by the "writable" contract, but note it in the Safety text.
- Proposed fix: in `mkpdf_document_new`, set `*out = null_mut()` right after the null check, before creating the handle. Do the same for later functions that return handles or buffers (FFI-001: set the pointer to null and length to 0 first). Add `out` alignment to the `# Safety` text.
- Disposition: rust-core, with the next FFI function. Proposed date: 2026-12-14.

### SEC-F-11 (low): aliasing, data races and reentrancy

- Evidence: `lib.rs:197`, `lib.rs:226` (`doc.as_mut()` creates `&mut MkpdfDocument`), `lib.rs:247` (`as_ref()`).
- Analysis: creating `&mut` from a raw pointer is sound only if the caller guarantees exclusivity for the call. That is the documented contract (`docs/api/c-abi.md` Threading), and it is the normal approach. The references live only inside the `guard` closure, so there is no long-lived `&mut`, and no reference escapes. A concurrent `page_count` (`&`) and `set_title` (`&mut`) on the same handle is a data race and UB; there is no detection. That is accepted by contract today and appears as T-07.
- Open point for FFI-001: if output is written through a caller callback, the callback could call back into the same handle while the `&mut` is live, which breaks aliasing rules. The streaming ADR must forbid it or the implementation must detect it (for example a `busy` flag in the handle set on entry and cleared on exit with a guard object, returning an error instead of UB).
- Disposition: document the reentrancy rule in ADR-0012 or the streaming ADR, and decide whether a `busy` flag is worth it (it also catches cross-thread misuse in many cases, at no real cost). Owner: architect and interop. Proposed date: before FFI-001.

### SEC-F-12 (low): thread-local error text

- Evidence: `lib.rs:74-94`, `lib.rs:131-146`.
- Analysis: the implementation is sound. It uses `try_with` and `try_borrow_mut`, so it does not panic during thread teardown. The `CString` is sanitised for NUL bytes, so `CString::new` cannot fail. The returned pointer is into the thread-local `CString`. It remains valid until the next `set_last_error` or `clear_last_error` on the same thread, which happens on any later guarded call, as documented. Because `mkpdf_last_error_message` does not itself clear or replace the text, reading it twice returns the same pointer, which is harmless.
- Caveats: (1) A runtime that moves a call between OS threads (Go goroutines without `runtime.LockOSThread`, .NET async continuations, Node worker handoffs) can read null or another call's text. This is a correctness and information-mix-up issue, not memory unsafety, but D-06 adds a Go SDK, so the SDK docs must say how to pin the thread. (2) A `thread_local!` with a destructor (`CString`) in a `cdylib` registers a destructor with the platform. On glibc this keeps the library mapped, on Windows it depends on the TLS callback, and unloading the library while a thread still has an error set has not been tested. (3) The message stays alive in the thread until overwritten. Messages contain only static text and constants today (T-27). They must never echo caller strings.
- Proposed fix: document the pinning requirement in `docs/api/c-abi.md`, add an unload test to the SDK suites, consider adding a `mkpdf_last_error_copy(buf, cap, &len)` variant before ABI 1.0 for managed runtimes.
- Disposition: interop. Proposed date: before the Go SDK starts.

### SEC-F-13 (low): functions that bypass `guard`

- Evidence: `rules/rust-engineering.md` item 5 says every exported function goes through `guard`. `docs/api/c-abi.md:45` says "Each exported function catches it". The code does not do so for `mkpdf_abi_version` (`lib.rs:115`), `mkpdf_version` (`lib.rs:121`), `mkpdf_last_error_message` (`lib.rs:134`, uses `catch_unwind` directly) and `mkpdf_document_free` (`lib.rs:173`, uses `catch_unwind` directly because it returns nothing).
- Analysis: I found no way for these to panic. `mkpdf_abi_version` is a constant shift of two small constants. `mkpdf_version` returns a pointer to a `static` `&CStr` built in a const context (`CStr::from_bytes_with_nul` on `concat!(env!("CARGO_PKG_VERSION"), "\0")`). The fallback arm `c"unknown"` covers a version string with an interior NUL. The static is valid for the life of the process, with the usual caveat that the pointer dangles if the library is unloaded. `mkpdf_last_error_message` wraps its body in `catch_unwind`, and uses `try_with` and `try_borrow`. So the behaviour is correct. But the documentation and the rule are not accurate.
- Proposed fix: change the rule and the ABI doc to say "every function that can fail or allocate" and list the exceptions with the reason, or wrap all of them. The `free` function not setting an error text on a caught panic is acceptable (it returns `void`), but say so in the doc. Note that `catch_unwind` in `free` swallows a panic in `Drop` silently and leaks the allocation, and that is acceptable.
- Disposition: architect and docs. Proposed date: 2026-12-07.

### SEC-F-14 (medium): panic strategy not decided or enforced

- Evidence: `docs/requirements.md` OQ-12 (open), threat model note 1. `Cargo.toml:34-38` sets no `panic` key, so the default (`unwind`) applies. Nothing prevents someone adding `panic = "abort"` later, and nothing tests that `catch_unwind` is effective in the shipped artefact. A cdylib or staticlib built by a downstream with `-C panic=abort` would also make every `guard` pointless.
- Analysis: since Rust 1.81, an unwinding panic that reaches an `extern "C"` boundary aborts the process, so the risk without `guard` is a host crash, not silent UB. With `guard` and `unwind`, a bug becomes `MKPDF_ERR_INTERNAL`. With `abort`, a bug aborts the host process, which is a denial of service for a server-side host. Both are defensible. The project must pick one, record it, and test it.
- Proposed fix: ADR-0011 should (a) choose, (b) pin `panic = "unwind"` explicitly in `[profile.release]` and `[profile.dev]` if unwind is chosen, with a comment, (c) add a CI check that fails if the setting changes (for example a test or script that greps the manifest), (d) document the consequence for SDK authors, and (e) say that stack overflow and out-of-memory abort the process in both cases (T-10, SEC-F-02). I did not review ADR-0011, because it is being drafted by another agent.
- Disposition: accept ADR-0011 after security review. Owner: architect. Proposed date: before v0.1.0, and no later than 2026-12-14.

### SEC-F-15 (low): hand-written header

- Evidence: `include/mkpdf.h`, `docs/api/c-abi.md:64-66`, `rules/interoperability.md` item 8.
- Analysis: I compared the header with the Rust signatures one by one: names, argument order and types (`size_t` to `usize`, `double` to `f64`, `int32_t` to `i32`, pointer constness) all match. The constants match. There is no mechanical check, so a future edit can drift. A wrong signature in the header is UB in every caller.
- Proposed fix: generate with cbindgen and diff in CI (FFI-001), or add a C compile test that includes the header and links against the library, and a Rust test that checks the constant values against the header text.
- Disposition: devops and interop. Proposed date: FFI-001.

### SEC-F-16 (low): floating toolchain and tool versions

- Evidence: `rust-toolchain.toml` (`channel = "stable"`), `ci.yml:20, 34, 49, 64` (`@stable`), `ci.yml:65` (unpinned `cargo-audit`). `Cargo.lock` is committed (3 packages), and `--locked` is not used in the CI commands, so a lockfile change would pass silently.
- Risk: builds are not reproducible, and a new stable release can change behaviour or lints (`RUSTFLAGS: -D warnings` at `ci.yml:13` turns a new warning into a failed build, which is good for hygiene but unpredictable). For release artefacts this matters for the release gate.
- Proposed fix: pin the release toolchain to an exact version, use `--locked` in every cargo command in CI, and update the pins through reviewed pull requests.
- Disposition: devops. Proposed date: before the first release.

### SEC-F-17 (low): lint configuration in the FFI crate

- Evidence: `lib.rs:9-10` has `#![allow(unsafe_code)]` and `#![allow(clippy::missing_safety_doc)]`. The workspace lints (`Cargo.toml:16-32`) do not include `clippy::undocumented_unsafe_blocks`. The test module has an `unsafe` block without a `SAFETY:` comment (`lib.rs:284-308`, `lib.rs:313-324`, `lib.rs:330-349`).
- Analysis: every non-test `unsafe` block has a `SAFETY:` comment today, and every `unsafe extern "C"` function has a `# Safety` section, so the allow of `missing_safety_doc` is not needed and hides a future omission. The `SAFETY:` comments in `set_title` and `add_page` say "caller guarantees liveness and exclusivity", which is accurate.
- Proposed fix: remove the `missing_safety_doc` allow, enable `clippy::undocumented_unsafe_blocks = "deny"` and `clippy::multiple_unsafe_ops_per_block = "warn"` for the FFI crate (they are stable lints), and replace the blanket `allow(unsafe_code)` with `#[allow(unsafe_code)]` on the items that need it if that is practical. Add `SAFETY:` comments in the tests or accept test blocks as exempt by a written rule.
- Disposition: rust-core. Proposed date: 2026-12-14.

### SEC-F-18 (low): build paths in release artefacts

- Evidence: macOS arm64 release `cdylib` has no match for the home path (checked with `strings`). Linux and Windows artefacts are not examined. `Cargo.toml:38` sets `debug = "line-tables-only"`, which embeds source paths in debug info.
- Proposed fix: build release artefacts with `--remap-path-prefix` for the workspace and the cargo registry, strip symbols in the shipped artefact, and check with `strings` in the release pipeline. Related to determinism (ADR-0007) and T-15.
- Disposition: devops, before the first release.

### SEC-F-19 (low): test gaps at the boundary

- Evidence: `lib.rs:257-358`. The tests cover happy path, null pointers, invalid UTF-8, NaN and `guard`. `rules/testing.md` item 2 asks for tests that call the functions "the way a C caller would"; there is no C test, and no `tests/` directory in the FFI crate.
- Missing: `len = usize::MAX` (SEC-F-01), a limit test for strings (SEC-F-02), a panic injected through a real exported function (SEC-F-03), a thread test (error text is per thread: set an error on thread A and check thread B sees none), NUL inside a title (documents that interior NUL is accepted), a test that `mkpdf_version` matches the header and `ABI_MAJOR`, a MAX_PAGES test through the FFI, and the C compile test from SEC-F-15.
- Disposition: qa. Proposed date: with each fix above.

## Review of the threat model

Changes made to `docs/security/threat-model.md` in this review (the file was edited, nothing was removed):

- Status line and note 6: now says one review by the Security Engineer role, no independent review, no risk acceptance.
- Statuses corrected: T-01 was "Implemented, tested", now "Tested for `guard` only", because the test does not exercise an exported function and Miri has not run. T-03 was "Open (accepted risk pending decision)", which is wrong because no risk has been accepted, now "Open. Needs an owner decision". T-12 now records that `cargo deny` passed but had nothing to check, and `cargo audit` is not installed. T-13 now names the unpinned actions. T-05 and T-06 name their tests. T-09 notes that a downstream build can drop the overflow checks.
- Threats added: T-18 (header drift), T-19 (hostile or wrong `len`), T-20 (handle after panic), T-21 (thread migration and unload), T-22 (uninitialised out parameters), T-23 (output buffer allocator mismatch, FFI-001), T-24 (reentrancy through callbacks), T-25 (no disclosure channel), T-26 (package name squatting and dependency confusion, OQ-02), T-27 (content in freed memory, error messages and logs), T-28 (CLI file handling, a new boundary TB-6).
- T-10 widened to include stack overflow, which aborts and cannot be caught.
- T-04 notes that allocation failure aborts the process.

Statuses I could not check: T-14 and T-15 (no PDF output exists), T-16, T-17 (deferred). T-07 and T-08 are as documented.

## Review of the CI workflow

See SEC-F-06, SEC-F-07 and SEC-F-16. Summary: permissions and triggers are good. There are no secrets. Actions and tools are not pinned. The Miri and MSRV jobs probably do not use the toolchain they name. The `audit` job installs `cargo-audit` from source on every run (slow, unpinned). There is no job that diffs the header, runs the C test, or runs sanitizers. There is no release workflow yet, so signing, provenance and SBOM are not reviewed.

## Conditions

| ID | Condition | Evidence needed | Before |
|---|---|---|---|
| C-1 | Fix SEC-F-01 and SEC-F-02 (length check, string limit) with tests | Commit and test names, review by security | FFI-001 merges |
| C-2 | ADR-0011 accepted after security review; `panic` setting pinned and checked (SEC-F-14, SEC-F-03) | ADR text, CI check | v0.1.0 |
| C-3 | A green CI run on all jobs, including Miri on nightly (SEC-F-05, SEC-F-06), with the log kept | CI run URL, Miri version, result | Closing GATE-1 fully |
| C-4 | Pin all actions and tools by SHA or version, add Dependabot, CODEOWNERS and branch protection (SEC-F-07, SEC-F-16) | Workflow diff, repository settings | First release |
| C-5 | Owner decision on SEC-F-04 (handle validation), written in the repository | Signed-off ADR or a note in `docs/requirements.md` | v0.2.0 planning |
| C-6 | Owner sets up the private reporting channel (SEC-F-09) | Updated `SECURITY.md` | Repository public |

## Statement on risk acceptance

No risk is accepted by this review. No finding is marked resolved, because no fix exists and no evidence of one has been seen. The Security Engineer cannot accept a risk. Only the project owner can, in writing, in the repository. Findings may only move to "Resolved" in a later revision of this file, with the commit and test or run evidence named next to each.

## Next review

Re-review when any of these happen: FFI-001 (buffers or callbacks) lands, the first dependency is added, the writer lands, ADR-0011 or ADR-0012 is accepted, or CI has run once. Any parser needs a new threat model section and fuzz targets first.

## Re-review: set_title fix (2026-11-23)

- Reviewer: Security Engineer (agent role). Single reviewer, not an independent second review.
- Scope: the uncommitted change to `crates/mkpdf-ffi/src/lib.rs`, `crates/mkpdf-core/src/model.rs` and `crates/mkpdf-core/src/lib.rs` on top of commit `a57d899`, which addresses SEC-F-01 and SEC-F-02. Other uncommitted edits (docs, agents, CI) are not reviewed here. I read `docs/api/c-abi.md`, `docs/api/rust-api.md`, `docs/security/threat-model.md` and `include/mkpdf.h` only to check consistency with the code.
- The only file this re-review edits is this one. No code was changed and nothing was committed.

### What was run (2026-11-23, macOS arm64, local, rustc 1.100.0 Homebrew, clippy 0.1.100)

| Check | Result |
|---|---|
| `cargo fmt --all --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed, no warnings. I touched the two changed source files first so that clippy re-checked them instead of using a cached result |
| `cargo test --workspace --locked` (debug) | Passed: `mkpdf-core` 8 tests, `mkpdf-ffi` 12 tests, `mkpdf-cli` 0, doc-tests 0 |
| `cargo test --workspace --locked --release` | Passed, same counts |
| `cargo deny check` (0.21.0) | Passed. No manifest or lockfile changed, so this confirms nothing new |
| Mutation test, in a throw-away copy under the session temp directory (deleted afterwards) | See "Do the tests prove the claims" below |

### What was NOT run

- Miri (not available here, SEC-F-05). The claim that no out-of-range slice is built rests on reading the code and on the standard library's debug UB checks, not on Miri.
- CI (has never run), Linux, Windows, 32-bit and wasm32 targets, MSRV 1.85.
- Sanitizers, fuzzing, a C caller test, `cargo audit` (not installed).

### 1. Order of checks and the SAFETY comment

- There is one `from_raw_parts` call in non-test code (`lib.rs:221`). Every path to it passes, in this order: `doc` null check (206), `len == 0` gives `&[]` with no pointer use (207), `utf8` null check (209), `len > isize::MAX` gives `INVALID_ARGUMENT` (211), `len > MAX_TITLE_BYTES` gives `LIMIT_EXCEEDED` (214). So on the only path that builds the slice, `0 < len <= 4096`. Verified.
- `isize::MAX as usize` is a lossless cast of a positive constant. With `MAX_TITLE_BYTES = 4096` the `isize::MAX` check is redundant on every target (`isize::MAX >= 32767`). It only selects a different status code, and it protects a future increase of the limit. That is acceptable.
- The SAFETY comment is true as far as it goes: the pointer is non-null, `u8` needs no alignment, the size bound is right, the bytes are only read, and the slice does not outlive the call (`str::from_utf8` borrows it and `Document::set_title` copies it with `to_owned`). It leaves out three conditions of `from_raw_parts` that come from the caller: the bytes must be initialised, they must lie in one allocation, and the caller must not change them during the call. See SEC-F-21.
- The non-test `unsafe` count is still 6. No new `unsafe` block was added outside tests.

### 2. Error text and status codes

- New messages: `"`len` exceeds isize::MAX"` (FFI), `"title exceeds the maximum length"` (FFI), `"title exceeds {MAX_TITLE_BYTES} bytes"` (core, constant only). All are static or built from a constant. None contains caller input. The new `Failure` variants take `&'static str`, so the type system prevents echoing a runtime string through them. Verified.
- Mapping: `Failure::InvalidArgument` gives 2, `Failure::LimitExceeded` gives 4, and core `ErrorKind::LimitExceeded` gives 4 as before. This matches `include/mkpdf.h` (2, 4) and the status table in `docs/api/c-abi.md` (2 "value outside its domain", 4 "a documented resource limit was exceeded").
- Gap: code 4 is defined as a *documented* limit, but the title limit is not documented in `docs/api/c-abi.md` or in the header (SEC-F-20).
- Precedence not documented: a null `utf8` with `len = usize::MAX` returns `NULL_POINTER`, not `INVALID_ARGUMENT` as the new `# Safety` text implies. It is safe either way, but the contract text should say which check wins (SEC-F-21).

### 3. Core limit

- `title.len()` on `&str` is the UTF-8 byte length. `> MAX_TITLE_BYTES` rejects, `== MAX_TITLE_BYTES` is accepted. The check runs before `to_owned`, and the assignment is the last statement, so the old title is kept on failure. Verified by reading and by mutation.
- `MAX_TITLE_BYTES = 4096` is below the old proposal of 64 KiB. Encoded as UTF-16BE with a BOM, a 4096-byte UTF-8 title is at most 8194 bytes, well under the 32 767-byte string limit in PDF 1.7 Annex C. I see no security objection to the value.
- The signature changed from `impl Into<String>` to `impl AsRef<str> -> Result<(), Error>`. That is a breaking Rust API change, acceptable in 0.x, but `docs/api/rust-api.md` still shows the old signature and "Any Unicode string" (SEC-F-20).
- The rustdoc of `MAX_TITLE_BYTES` points to threat model T-10 (decompression bombs). The right row is T-04 (resource exhaustion, strings) (SEC-F-20).

### 4. Panic, overflow, leak

- No new arithmetic in non-test code, no indexing, no `unwrap`. The only allocations on the new paths are the short static message strings and the title copy of at most 4096 bytes. All early returns happen before any allocation, so nothing leaks. A failing test assertion would leak its handle, which is test-only.
- Allocation failure still aborts. That is now stated in `docs/api/c-abi.md` ("Out-of-memory and stack overflow abort the process in either mode", in the uncommitted doc edit). The input-driven allocation is now bounded at 4096 bytes, which removes the large-allocation abort that SEC-F-02 described.
- Not new, still open: the `&mut MkpdfDocument` is created before the arguments are validated (architect review FFI-CR, SEC-F-11). It does no harm here, because nothing on the error paths can panic or touch the document.

### 5. Do the tests prove the claims

I changed the code in a copy of the tree and ran the tests against each change:

| Mutation | Debug `cargo test` | Release `cargo test` |
|---|---|---|
| M1: build the slice first, then run both length checks (slice built but never read) | Caught. The standard library's `from_raw_parts` precondition check aborts the test binary (SIGABRT) in `set_title_rejects_len_just_over_isize_max_without_reading` | **Not caught.** All 12 tests pass |
| M2: remove the FFI `MAX_TITLE_BYTES` check (rely on the core) | Caught by `set_title_rejects_len_isize_max_as_limit_without_reading` | Caught |
| M3: remove the FFI `isize::MAX` check | Caught by both `..._usize_max_...` and `..._just_over_isize_max_...` (wrong status code) | Caught |
| C5: remove the core check | Caught by 2 core tests. The FFI tests still pass, because the FFI check hides it. That is expected | n/a |
| C6: count `chars()` instead of bytes | Caught by `title_limit_counts_bytes_not_chars` | n/a |
| C7: off by one (`>=`) | Caught by 2 core tests | n/a |
| C8: assign the title before the check | Caught by 2 core tests (old title not kept) | n/a |

Conclusions:

- Byte counting, the boundary, keeping the old title, the status codes and "no input echo" are all proven by tests that fail when the property breaks.
- "The check runs before the slice is built" is proven **only in debug builds**, through the standard library's UB checks, and the failure is a crash of the test binary rather than a failed test. The comment on `set_title_with_len` ("the test only passes if the length check runs before any slice is built") is therefore too strong (SEC-F-21). CI runs `cargo test` in debug, so a regression would still show up there. Miri would catch it in any build.
- The "no echo" assertions check that a marker substring is absent. Since the messages are `&'static str`, that is enough today. An exact-match assertion would be stronger.
- Not tested: null `utf8` with a huge `len`; null `doc` for `set_title`; a non-null pointer with `len = 0`; a title containing U+0000 (FFI-CR-14); `MAX_PAGES` through the FFI (the new `max_pages_is_enforced` test covers the core only).

### 6. Dispositions

| ID | Before | After | Evidence | What is still needed |
|---|---|---|---|---|
| SEC-F-01 | Open | **Fixed in the working tree, not committed** | `lib.rs:207-221`; tests `set_title_rejects_len_usize_max_without_reading`, `set_title_rejects_len_just_over_isize_max_without_reading`, `set_title_rejects_len_isize_max_as_limit_without_reading`; mutations M1 (debug), M3 | Commit the change with the tests unchanged, then record the commit hash here and mark it Resolved. Not marked Resolved now, because this file requires a named commit. Miri confirmation stays under SEC-F-05 |
| SEC-F-02 | Open | **Partly fixed, not committed** | `model.rs:10-12, 87-97`, `lib.rs:214-215`; core tests `title_at_limit_is_accepted`, `title_over_limit_is_rejected_and_old_title_kept`, `title_limit_counts_bytes_not_chars`; FFI tests `set_title_over_limit_is_rejected_and_keeps_previous_title`, `set_title_at_limit_is_accepted`, `set_title_failures_keep_previous_title`; mutations C5 to C8, M2 | The fix proposed for SEC-F-02 included documenting the limit in `docs/api/c-abi.md`, and that has not been done. Also needed: the header, `docs/api/rust-api.md`, threat model T-04 and T-19, and the CHANGELOG (SEC-F-20). Then commit |
| SEC-F-19 | Open | **Partly addressed, stays Open** | `len = usize::MAX` test and string-limit tests now exist. A core `MAX_PAGES` test exists | Still missing: panic through a real exported function, thread test for error text, interior NUL, `mkpdf_version` against the header, `MAX_PAGES` through the FFI, C compile test, a `tests/` directory for the FFI crate |

Condition C-1 is not met yet. It needs the commit, SEC-F-20 fixed and this review updated with the commit hash.

### New findings

#### SEC-F-20 (low): the limit and the new behaviour are not documented

- Evidence: `docs/api/c-abi.md:63` lists `set_title` with "Set the title" and no limit or error codes. `include/mkpdf.h:42-43` has no limit and no constant. `docs/api/rust-api.md:13` shows `fn set_title(&mut self, impl Into<String>)` "Any Unicode string" and `:30` lists no `MAX_TITLE_BYTES`. `docs/security/threat-model.md` T-04 still says "No limit on title length" and T-19 says "Open". `model.rs:10-11` refers to T-10 instead of T-04. `CHANGELOG.md` does not mention the limit.
- Risk: SDK authors cannot know the limit or that `set_title` can now return 2 or 4, so wrappers may treat the error as a bug, or truncate in a way that splits a UTF-8 sequence. A `LIMIT_EXCEEDED` with no documented limit goes against the meaning of code 4.
- Proposed fix: in `c-abi.md`, state that the title is at most 4096 bytes of UTF-8 (`LIMIT_EXCEEDED`), that `len > isize::MAX` gives `INVALID_ARGUMENT`, which check comes first, and that the old title is kept on failure. Add `#define MKPDF_MAX_TITLE_BYTES 4096u` (or a comment) to the header and a test that the value matches the Rust constant (that also helps SEC-F-15). Update `rust-api.md`, the threat model rows and the changelog. Fix the T-10 reference.
- Disposition: Owner docs and interop, with rust-core for the comment. Due: in the same commit as the fix, and no later than 2026-12-07.

#### SEC-F-21 (low): SAFETY text is incomplete; one test comment claims too much

- Evidence: `lib.rs:194-197` and `lib.rs:217-220` do not mention that the bytes must be initialised, lie in one allocation, and not change during the call. The `# Safety` text says `len > isize::MAX` gives `INVALID_ARGUMENT`, but with a null `utf8` it gives `NULL_POINTER`. `lib.rs:382-383` says the test passes only if the check comes first, and mutation M1 shows this is false in release builds.
- Risk: no memory-safety defect today. The text is the contract that SDK authors and later maintainers rely on, and an incomplete statement weakens later reviews.
- Proposed fix: add the three caller conditions to the `# Safety` text and the SAFETY comment, and state the check order. Add `const _: () = assert!(MAX_TITLE_BYTES <= isize::MAX as usize);` so that the "below `isize::MAX`" claim is checked by the compiler. Reword the test comment to say that the order is checked by the standard library's debug UB checks and by Miri. Add the missing tests listed in item 5.
- Disposition: Owner rust-core (code), qa (tests). Due 2026-12-07.

### Risk acceptance

None. No risk is accepted by this re-review, and none was asked for. No critical or high finding was found, so nothing is escalated.
