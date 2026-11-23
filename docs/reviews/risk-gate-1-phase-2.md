# GATE-1 phase 2: independent risk review of the architecture review

- Reviewer: Risk Reviewer (agent role). Second review of `docs/reviews/gate-1-architecture.md`, ADR-0011, ADR-0012, and the architect's edits to `docs/architecture.md`, `docs/api/c-abi.md`, `docs/api/rust-api.md`. I did not write any of those files.
- Model: this session's configuration text reports `claude-fable-5-1` (`.opencode/agents/risk-reviewer.md:4`). That is self-reported and unverified from inside the session. The architecture review states it may have run on `claude-sonnet-5-5` rather than the `claude-opus-5-5` now configured in `.opencode/agents/architect.md:4`. Phase 1 (`risk-gate-1-phase-1.md:3`) reports `claude-sonnet-5-5`. None of these attributions can be checked from the repository.
- Date: 2026-11-23
- Repository state: remote `main` at `a57d899` (CI run 45097339864). Local working tree has uncommitted changes. I could not run `git diff`, so the extent of the working tree is known only by comparing local files with the remote copies fetched through the GitHub contents API.
- Marking: **[V]** verified (I read the file and line, or saw API output), **[U]** unverified, **[A]** assumed from general knowledge, not from a run.
- Phase 1 (`docs/reviews/risk-gate-1-phase-1.md`) and the register (`docs/risk/register.md`) belong to the concurrent phase 1 session. I did not edit them. Proposed register changes are at the end.

## Recommendation

**PASS WITH CONDITIONS** for phase 2. Not a clean close of GATE-1.

- The architecture review is substantive. Every file-and-line citation I checked is correct (22 of 22 findings have evidence that exists where cited). Its severities are defensible. Its doc edits match the code where they describe the code.
- Its main defects: (1) it says "CI has never run" when a run existed with a failed Miri job, so ARCH-F-17 and AC-7 are stale; (2) every artefact it produced, and every owner decision it relies on (D-08, D-09, FFI-003, DEC-001), exists only in the uncommitted working tree; (3) ADR-0012 fixes a pointer-shaped buffer handle while the handle shape is exactly what the still-unwritten D-08 ADR must decide, and the roadmap orders DEC-001 before that ADR; (4) the conditions and roadmap put the writer (PDF-001) behind FFI-003's implementation and a working Miri job, which is not a design dependency.
- I ran no command. All of the architect's "What was run" table is [U] for me (RISK-F-P2-01). This is the second consecutive Risk Reviewer session in which the permitted shell commands were denied.
- No critical finding. No high finding. 5 medium, 9 low.
- I accept no risk. Only the project owner can, in writing in the repository.

## Limits of this review

- Shell: `cargo test --workspace`, `cargo test`, `git log`, `git log --oneline -15`, `git diff --stat`, `git diff --name-only` were all rejected with "Permission denied: shell" [V]. The agent definition allows these (`risk-reviewer.md:18-32`) [V]. So: `cargo fmt --check`, `cargo clippy`, `cargo test`, `cargo deny`, `cargo doc`, `nm`, cbindgen and the C smoke program were **not run** by me. The last three are outside my allowlist anyway.
- `gh run` was not attempted because every shell call was denied. CI data below comes from the unauthenticated public API (run list, job list with step conclusions, compare, file contents at `a57d899`) [V]. Job logs were not read.
- I did not re-review the `unsafe` code. That belongs to the Security Engineer. I read it to check the architect's citations.

## What was examined

- [V] `AGENTS.md`, all seven files in `rules/`, `.opencode/agents/risk-reviewer.md`, `docs/requirements.md`, `docs/roadmap.md`.
- [V] `docs/reviews/gate-1-architecture.md` (250 lines), `docs/reviews/gate-1-security.md` (239 lines), `docs/reviews/risk-gate-1-phase-1.md`, `docs/risk/register.md` (read only).
- [V] ADR-0001 to ADR-0012 and `docs/adr/README.md`.
- [V] `docs/architecture.md`, `docs/api/c-abi.md`, `docs/api/rust-api.md`, `docs/testing.md`, `docs/dependencies.md`, the T-01/03/04/07/10/15/20/23/24 rows of `docs/security/threat-model.md`.
- [V] `crates/mkpdf-ffi/src/lib.rs` (358 lines), `crates/mkpdf-core/src/{lib,model,error}.rs`, `crates/mkpdf-cli/src/main.rs`, `include/mkpdf.h`, root `Cargo.toml`, `crates/mkpdf-ffi/Cargo.toml`, `crates/mkpdf-cli/Cargo.toml`, `deny.toml`, `rust-toolchain.toml`, `clippy.toml`, `.github/workflows/ci.yml`.
- [V] GitHub API: workflow runs, jobs and steps of run 45097339864, last 5 commits, compare `59df15e...a57d899`, and remote copies at `a57d899` of `Cargo.toml`, `docs/architecture.md`, `docs/api/c-abi.md`, `docs/adr/README.md`, `docs/requirements.md`, `docs/roadmap.md`; existence checks for `docs/reviews/gate-1-architecture.md` and `docs/adr/0011-panic-strategy.md` (both HTTP 404 on the remote).

## Verification of the architecture review

### "What was run" table (`gate-1-architecture.md:38-54`)

| Claim | My status | Note |
|---|---|---|
| `cargo fmt`, `cargo clippy`, `cargo test` passed, 4 + 6 tests, 0 doc tests | [U] | Shell denied. Consistent with CI job `lint` and the four `test` jobs succeeding at `a57d899` [V, API], and with 10 `#[test]` functions in the code (4 in `model.rs:113-147`, 6 in `lib.rs:269-357`) [V]. No doc examples exist in any doc comment [V by reading], so 0 doc tests is consistent |
| `cargo deny check` passed with 7 `license-not-encountered` warnings | [U] run, [A] count | See RISK-F-P2-06 |
| `cargo doc` 2 warnings | [U] | The private link `[`guard`]` at `lib.rs:4` exists [V]. Library name `mkpdf` (`mkpdf-ffi/Cargo.toml:12`) and binary name `mkpdf` (`mkpdf-cli/Cargo.toml:12`) collide [V] |
| `nm` shows 8 symbols | [U] | 8 `#[unsafe(no_mangle)] extern "C"` functions in `lib.rs` [V]; names match the header [V] |
| cbindgen output differs (`ABI_MAJOR` vs `MKPDF_ABI_MAJOR`) | [U] | Rust constants are `ABI_MAJOR`/`ABI_MINOR` (`lib.rs:19,21`) and the header macros are `MKPDF_ABI_MAJOR`/`MKPDF_ABI_MINOR` (`mkpdf.h:15-16`) [V], so a default cbindgen run would differ [A] |
| C program compiled and ran; C++17 parse ok | [U] | Header has `extern "C"` guards and only C types [V] |
| `mkpdf --version` prints `mkpdf 0.0.1`; non-UTF-8 argument panics with exit 101 | [U] | `main.rs:8` uses `std::env::args()`, which panics on non-UTF-8 [A, std documentation]. Version is `0.0.1` (`Cargo.toml:6`) [V] |
| `catch_unwind` exit 0 vs 134; `cfg(panic = "abort") compile_error!` fires only with abort | [U] | Standard language behaviour [A] |
| Release build sizes | [U] | Not material; the ADR says so itself |

### Findings ARCH-F-01 to ARCH-F-22: evidence check

| Finding | Evidence cited | Checked | Severity view |
|---|---|---|---|
| ARCH-F-01 | ADR-0003:9,21; D-08; `lib.rs:153-182` | [V] all. ADR-0003:21 says UB plus "open option for GATE-1"; D-08 says build generation-checked handles; handles are `Box::into_raw` pointers | Agree, medium |
| ARCH-F-02 | `roadmap.md:71,55-56` | [V]. FFI-001 depends on "PDF-002, API-002"; FFI-003 depends on DEC-001; D-08 says FFI-003 before FFI-001 | Agree, medium |
| ARCH-F-03 | ADR-0004:7-10; `lib.rs:19-21`; `c-abi.md:3` | [V]. The ambiguity is real: "change ... raises the major" and "frozen at v1.0.0" | Agree, medium |
| ARCH-F-04 | `Cargo.toml:34-38` no `panic` key; `lib.rs:96-111` | [V]. Only `[profile.release]` exists, no `panic` | Agree, medium |
| ARCH-F-05 | `mkpdf.h:15-16`; `c-abi.md:64-66` | [V] header. The `c-abi.md:64-66` cite is stale after the architect's own edit: the header statement is now at `c-abi.md:71-73` [V] | Agree, medium |
| ARCH-F-06 | OQ-13, `c-abi.md` "Not yet specified", T-23, T-24 | [V] | Agree, medium |
| ARCH-F-07 | `model.rs:8,86`; no limit test; `model.rs:74`; `lib.rs:190-210` | [V]. No test in `model.rs` or `lib.rs` reaches `MAX_PAGES`; `set_title` has no bound | Agree, medium |
| ARCH-F-08 | ADR-0007:3,9; OQ-10 | [V] | Agree, medium |
| ARCH-F-09 | `lib.rs:115,121,134,173` | [V]. Four functions do not use `guard`. The security review says "three" (SEC-F-13 title) and lists four; the architect's count is right | Agree, low |
| ARCH-F-10 | `lib.rs:79,96-101` | [V]. Only `guard` clears the error text (`lib.rs:98-100`); `mkpdf_last_error_message` does not | Agree, low |
| ARCH-F-11 | ADR-0006:16; `model.rs:61-64` | [V]. `Document` has `title` and `pages` only | Agree, low |
| ARCH-F-12 | ADR-0009:9; `ci.yml:45-52` | [V]. Job exists. It has now **run and passed** (RISK-F-P2-02) | Agree, low |
| ARCH-F-13 | ADR-0010:9,11-12 | [V] | Agree, low |
| ARCH-F-14 | ADR-0008:8; `lib.rs:52-55` | [V] | Agree, low |
| ARCH-F-15 | `main.rs:8` | [V] line; panic [A] | Agree, low (CLI only has `--version` today) |
| ARCH-F-16 | `lib.rs:4`; the two `Cargo.toml` names; no doc job in `ci.yml` | [V] | Agree, low |
| ARCH-F-17 | `ci.yml:42,72`; `rust-toolchain.toml:2` | [V] lines. **"CI has never run" is false**: see RISK-F-P2-02 | Agree, medium; text stale |
| ARCH-F-18 | `Cargo.toml:16-32`; `lib.rs:10`; `deny.toml` | [V]. `indexing_slicing = "warn"` [V]; no `[bans].deny` [V]; `missing_safety_doc` allowed [V] | Agree, low |
| ARCH-F-19 | `rust-api.md`; derives | [V]. `Document` derives `Debug, Clone, Default, PartialEq` (`model.rs:60`); `Page` `Debug, Clone, PartialEq` (`:11`); `Error` `Debug, Clone, PartialEq, Eq` + `Display` + `std::error::Error` (`error.rs:14,41,47`); `ErrorKind` `Debug, Clone, Copy, PartialEq, Eq`, `#[non_exhaustive]` (`:4-5`). Signatures match the spec table one by one | Agree, low |
| ARCH-F-20 | OQ-03 text; `testing.md:11`; `dependencies.md:26`; `lib.rs:6`; deny count | [V] all four stale statements exist. `ci.yml:30` lists four runners [V]. Deny count: RISK-F-P2-06 | Agree, low |
| ARCH-F-21 | `docs/architecture.md` before and after | [V]. Remote copy at `a57d899` says "No architecture or security review has been done" and "a callback receives chunks" is gone locally; the local file has the new text | Agree, low |
| ARCH-F-22 | ADR README; rule 5 | [V]. All twelve Proposed | Agree, low |

### The architect's doc edits against the code

- `docs/api/c-abi.md:10` (outbound strings NUL-terminated, NUL replaced by `?`): matches `lib.rs:79,122-128,140` [V].
- `c-abi.md:42` (only status-returning calls clear the text): matches `lib.rs:98-100,134-146,173-182` [V].
- `c-abi.md:46-50` (four functions without `guard`, with reasons): matches `lib.rs:115,121,134,173` [V].
- `c-abi.md:56-65` function table: matches `mkpdf.h:29-49` and the Rust signatures [V].
- `c-abi.md:69` still says "Serialisation to a caller buffer or callback (FFI-001)" and `c-abi.md:73` still says "`cbindgen` is not installed here". The first is the wording the architect criticised in ARCH-F-02; the second contradicts `roadmap.md:96` and the architect's own cbindgen run. Both are in a file the architect edited [V]. RISK-F-P2-07.
- `docs/api/rust-api.md`: every signature and constant matches `model.rs` and `error.rs` [V]. `set_title` "Any Unicode string" is accurate and is the SEC-F-02 gap. `Send`/`Sync` follows from owned `String`, `Vec<Page>`, `f64` fields [V fields, A for the auto-trait inference].
- `docs/architecture.md:54`: "Resource limits are constants in the core (page size, page count). **Every input-driven allocation gets one.**" The second sentence is not true today (title string, ARCH-F-07, SEC-F-02). The architect says the "Key properties" were checked and kept [`gate-1-architecture.md:228`]. This is planned work stated as a present property [V]. RISK-F-P2-07.
- `docs/architecture.md:56`: wasm job "has never run". It ran and passed in run 45097339864 [V, API]. RISK-F-P2-02.

## Findings

| ID | Severity | Title | Disposition |
|---|---|---|---|
| RISK-F-P2-01 | medium | No command in the architecture review was independently re-run; the Risk Reviewer's permitted shell is denied for the second session running | Owner or Orchestrator fixes the permission; someone with shell reproduces the table |
| RISK-F-P2-02 | medium | Architecture review says "CI has never run"; a run exists with a failed `miri-ffi` job. ARCH-F-17, AC-7, ARCH-F-12, `architecture.md:56` and `testing.md:9,15` are stale | Architect re-baselines against run 45097339864 |
| RISK-F-P2-03 | medium | Every GATE-1 artefact and the owner decisions D-08 and D-09 exist only in the uncommitted working tree | Commit (owner or Orchestrator) before any condition is treated as recorded |
| RISK-F-P2-04 | medium | ADR-0012 fixes a pointer-shaped buffer handle before the D-08 ADR decides the handle shape; DEC-001 is ordered before that ADR, so an accepted ADR-0012 may need superseding at once | Reorder: D-08 ADR first, or make ADR-0012 shape-neutral |
| RISK-F-P2-05 | medium | `GATE-1` depends on "all above" including FFI-003 (implementation, Miri passing), and PDF-001 depends on GATE-1. The writer is blocked behind FFI handle-table work and a CI fix without a design reason | Orchestrator splits FFI-003 into ADR (gates GATE-1) and implementation (gates FFI-001 only) |
| RISK-F-P2-06 | low | `cargo deny` warning count: 7 (architect) vs 4 (security review). The file arithmetic supports 7. Neither run is reproducible from the repository | Re-run with shell, record output; security review corrects or explains |
| RISK-F-P2-07 | low | Statements in the architect's own edited docs are stale or planned-as-present (`architecture.md:54`, `c-abi.md:69,73`) | Architect or Docs agent corrects |
| RISK-F-P2-08 | low | ADR-0011 omits `panic_nounwind` aborts (std unsafe-precondition checks, debug assertions) from the list of uncatchable failures; this is exactly the SEC-F-01 path | Add to ADR-0011 item 5 before acceptance |
| RISK-F-P2-09 | low | ADR-0011 item 3's test-only panic hook "behind a feature never in a release build" has no proposed enforcement | State the mechanism (e.g. `cfg(test)`-only or a CI check that the feature is off) |
| RISK-F-P2-10 | low | ADR-0012 says T-23 is "closed by rules 1, 2 and 4"; rule 1 depends on FFI-003 existing. Also `rules/interoperability.md` item 2 lifetime wording does not cover `mkpdf_buffer_data` | Reword "closed when FFI-003 lands"; propose the rule amendment to the owner |
| RISK-F-P2-11 | low | ADR-0012 option B critique overstates ("heap overflow in the caller's buffer"); a correct implementation that honours `cap` cannot overflow. The conclusion still holds on CPU cost and TOCTOU | Soften the sentence; no change to the recommendation |
| RISK-F-P2-12 | low | The review's conditions (AC-1) require acceptance of ADRs the same agent wrote; the only independent check is the Security Engineer review, which has not happened | Keep AC-1; add "Risk Reviewer re-reads after Security Engineer review" |
| RISK-F-P2-13 | low | Roadmap FFI-003 text prejudges AC-3 ("the ABI minor version is raised") and AC-4 does not gate FFI-003 although FFI-003 changes the header | Orchestrator edits the roadmap after the owner decides AC-3; extend AC-4 to FFI-003 |
| RISK-F-P2-14 | low | Model attribution of the three reviews is self-reported and inconsistent across files | Record in each review that the model line is self-reported; owner decides if it matters |

### RISK-F-P2-01 (medium): nothing re-run

- Evidence: six shell attempts in this session rejected ("Permission denied: shell") [V]. Phase 1 reports the same (`risk-gate-1-phase-1.md:17`) [V]. The role's allowlist includes these commands (`risk-reviewer.md:18-32`) [V].
- Impact: the architecture review's table of runs, the security review's table, and the local evidence for ARCH-003 ("Done (macOS only)") have no independent reproduction. GATE-1 would close on single-source command evidence. This is a false-assurance risk in the process, not in the code.
- Disposition: the Orchestrator or owner fixes the agent's shell permission (the deny-all rule may be shadowing the allows), and someone with shell re-runs `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo deny check` and stores the output under `docs/reviews/`. Until then every "Passed" in both gate reviews stays [U].

### RISK-F-P2-02 (medium): "CI has never run" is false

- Evidence: `gate-1-architecture.md:58` "CI has never run. Nothing in `.github/workflows/ci.yml` is verified" [V]. GitHub API: run 45097339864, push to `main`, head `a57d899`, created 2026-11-23T01:56:20Z, conclusion `failure` [V]. Jobs: `lint`, `test` on all four runners (including `ubuntu-24.04-arm`), `msrv`, `wasm-core`, `deny`, `audit` all `success`; `miri-ffi` step "Run cargo miri test -p mkpdf-ffi" `failure` after the `dtolnay/rust-toolchain@nightly` step succeeded [V]. The review is dated the same day and states the working tree contained `gate-1-security.md` and other later edits (`:6`).
- Consequences: ARCH-F-17 point (4) (ARM runner availability unconfirmed) is answered: it exists [V]. ARCH-F-12's wasm job has run and passed (a build only) [V]. The architect's "probably overridden" toolchain theory now has a data point: Miri failed in about 11 s after the nightly step passed, which fits a stable override but is not proven (log not read) [U]. AC-7 is open and measurable now. `docs/architecture.md:56` and `docs/testing.md:9,15` are stale as well [V].
- Disposition: architect re-baselines ARCH-F-17, ARCH-F-12, AC-7 and the architecture doc against the run, with the run ID. Same instruction as phase 1 gave the security review (RISK-F-05). Not a defect in analysis, a defect in currency.

### RISK-F-P2-03 (medium): the gate exists only in the working tree

- Evidence: at `a57d899` the remote returns 404 for `docs/reviews/gate-1-architecture.md` and `docs/adr/0011-panic-strategy.md` [V]. Remote `docs/requirements.md` contains no `D-08` or `D-09`; remote `docs/roadmap.md` has no `FFI-003`; remote `docs/adr/README.md` ends at ADR-0010 [V]. Compare `59df15e...a57d899` touches only `Cargo.toml`, `LICENSE`, `SECURITY.md`, `docs/requirements.md` (2 lines) [V]. The architect states the review did not commit (`gate-1-architecture.md:6`) [V].
- Impact: `AGENTS.md` and `rules/security.md` item 9 require decisions and acceptances "in writing in the repository". An uncommitted file is not in the repository history. D-08 is the decision that changes the ABI, and it is not recorded anywhere durable. A session crash or a `git checkout` loses the gate. CI has not seen any of these files, so the lint, deny and link state of the new docs is untested.
- Disposition: the owner (DCO sign-off, D-04) commits the working tree as the GATE-1 record before any condition is considered met. I did not commit, and may not.

### RISK-F-P2-04 (medium): ADR-0012 vs the unwritten D-08 ADR

- Evidence: ADR-0012 sketch (`0012-output-buffer-ownership.md:47-55`) declares `typedef struct MkpdfBuffer MkpdfBuffer;` and `MkpdfBuffer **out`, i.e. a pointer-typed handle [V]. ADR-0012 rule 1 says "It is generation-checked under D-08" [V]. ARCH-F-01 says the D-08 ADR must settle "handle representation (a `uint64_t` token, or a pointer-typed token that is never dereferenced)" (`gate-1-architecture.md:104`) [V]. Roadmap: FFI-003 depends on DEC-001, and FFI-003's acceptance text contains the D-08 ADR (`roadmap.md:55-56`) [V]. AC-2 says the D-08 ADR must exist before FFI-003 starts, and that ADR-0012 is "re-read" against it [V]. `docs/adr/README.md:3`: accepted ADRs are not edited [V].
- Problem: the sequence the documents prescribe is accept ADR-0012 (DEC-001) → write the D-08 ADR → if D-08 picks an integer token, ADR-0012's sketch and rules 1 and 2 are wrong and ADR-0012 must be superseded immediately. "Re-read" is not a rule-compatible remedy for an accepted ADR. The dependency DEC-001 → FFI-003 is backwards for the part of FFI-003 that is a decision.
- Does ADR-0012 conflict with D-08? Not in intent. It conflicts in ordering and in the concreteness of the sketch.
- Disposition: either (a) write the D-08 ADR first (a new item, say DEC-002, that gates both DEC-001's acceptance of ADR-0012 and FFI-003), or (b) ADR-0012 states explicitly that the token type is decided by the D-08 ADR and shows the sketch as illustrative. The Solution Architect decides which; the owner accepts.

### RISK-F-P2-05 (medium): the writer is blocked behind FFI implementation

- Evidence: `roadmap.md:59` GATE-1 "Depends on: all above", which includes FFI-003 ("Tests cover each case, and Miri passes") and DEC-001 [V]. `roadmap.md:65` PDF-001 depends on GATE-1 [V]. FFI-003's Miri requirement depends on the failed `miri-ffi` job being fixed (AC-7, C-3) [V]. The architecture review's recommendation is explicit that it closes "the architecture part of GATE-1" only (`:12-15`) [V].
- Problem: PDF-001 is core-only work (`mkpdf-core`, `forbid(unsafe_code)`). It does not need the handle table. Chaining it behind FFI-003's implementation and behind a CI fix is a schedule risk with one owner (register R-17), not a design dependency. There is no circular dependency, but AC-2 ("D-08 ADR before FFI-003 **starts**") and FFI-003's acceptance criteria ("An ADR amends ADR-0003") disagree on whether the ADR is a precondition of FFI-003 or part of it.
- Disposition: Orchestrator splits FFI-003 into the ADR (Phase 0, gates GATE-1 together with DEC-001, AC-5, AC-6) and the implementation (Phase 1, gates FFI-001 only). PDF-001 then depends on the decisions, not on the FFI code. Owner approves the roadmap change (D-01).

### RISK-F-P2-06 (low): the `cargo deny` warning count

- Evidence: `deny.toml:11-20` allows eight licences [V]. All three workspace crates are `Apache-2.0` (`Cargo.toml:9`, inherited) [V]. `deny.toml` is unchanged between `59df15e` and `a57d899` (not in the compare file list) [V]. cargo-deny v2 emits one `license-not-encountered` per allowed licence not seen in the graph [A]. 8 − 1 = 7, which is the architect's count and the architect's list (MIT, ISC, Unicode-3.0, Zlib, Apache-2.0 WITH LLVM-exception, BSD-2-Clause, BSD-3-Clause) [V list, A arithmetic]. The security review's four (`gate-1-security.md:31`) does not follow from the file unless the tool treats the two Apache and the two BSD identifiers specially, which I cannot confirm.
- I could not run `cargo deny` (shell denied, and it is outside my allowlist).
- Disposition: not material to any decision. Someone with shell runs `cargo deny check 2>&1 | grep -c license-not-encountered` and the Security Engineer corrects line 31 or explains the count. Record the cargo-deny version with the result.

### RISK-F-P2-07 (low): stale or planned-as-present text in the architect's own edits

- `docs/architecture.md:54` "Every input-driven allocation gets one [limit]": false today for the title string (`model.rs:74`, `lib.rs:190-210`) [V]. The sentence should read "must get one" or name the gap.
- `docs/api/c-abi.md:73` "because `cbindgen` is not installed here": contradicted by `roadmap.md:96` and by the architect's own cbindgen run (`gate-1-architecture.md:48`) [V]. Phase 1 RISK-F-08 lists the same line.
- `docs/api/c-abi.md:69` "caller buffer or callback": the wording ARCH-F-02 objects to, kept in a file the architect edited [V].
- `gate-1-architecture.md:128` cites `c-abi.md:64-66` for the header statement, now at `:71-73` after the edit [V].
- Disposition: architect fixes in the next revision. Low, but the first one is the pattern the Risk Reviewer exists to catch.

### RISK-F-P2-08 (low): ADR-0011 and `panic_nounwind`

- Evidence: ADR-0011 item 5 and Context list out-of-memory and stack overflow as the aborts `catch_unwind` cannot stop (`0011-panic-strategy.md:18,28`) [V]. Since Rust 1.78 the standard library's unsafe-precondition checks (for example the `from_raw_parts` length and alignment check that SEC-F-01 discusses) are enabled in debug builds and abort through `panic_nounwind`, which no `catch_unwind` catches; the same is true of any `#[rustc_nounwind]` path [A, language knowledge, not run].
- Impact: the ADR's statement of what `guard` protects against is incomplete in exactly the direction that matters for the FFI crate. A debug build of the FFI crate with a wrong `len` aborts the host even with `unwind`. This supports the SEC-F-01 fix (check before the slice), which ADR-0011 should cite.
- Disposition: add the case to item 5, and cross-reference C-1. Security Engineer confirms during the ADR review.

### RISK-F-P2-09 and RISK-F-P2-10 and RISK-F-P2-11: ADR text points

- P2-09: ADR-0011 item 3 proposes a test hook "behind a feature that is never in a release build" (`:26`) [V]. No mechanism is proposed. A Cargo feature can be enabled by any downstream. Propose `#[cfg(test)]` plus an integration test in `tests/`, or a CI step that checks the release artefact does not export the hook symbol.
- P2-10: ADR-0012 Consequences "T-23 is closed by rules 1, 2 and 4" (`:91`) [V]. Rule 1 (stale pointer returns `NULL`) is true only after FFI-003 lands. Until then a double free of a buffer is UB like any handle. Reword to "closed when FFI-003 is implemented". Separately, `rules/interoperability.md` item 2 says borrowed pointers are "valid only until the next call on the same thread" [V]; `mkpdf_buffer_data` is valid until `mkpdf_buffer_free` (rule 2 of the ADR). The rule needs the owner to approve a wording that allows "or until the owning handle is freed, as documented per function".
- P2-11: ADR-0012 option B: "a bug would turn into a heap overflow in the caller's buffer" (`:32`) [V]. A write bounded by `cap` cannot overflow regardless of the size query; only an implementation that trusts the first size instead of `cap` can. The fair statement is: double serialisation cost, TOCTOU between calls, and a `cap`-mismatch error path every caller must handle. The recommendation of option A stands on those grounds.

### ADR-0011 and ADR-0012: assessment of the recommended options

- ADR-0011 (`unwind`, pinned, `compile_error!` under `abort`, CI check, real-function panic test, poisoned handle): sound. The alternatives are fairly compared; alternative C ("per crate") is correctly rejected as impossible for one artefact [A]. The claims about Rust ≥ 1.81 aborting on unwind out of `extern "C"`, Cargo ignoring `panic` for the `test` profile, and `wasm32-unknown-unknown` having no unwinding on stable are correct to my knowledge [A]. Gaps: P2-08, P2-09. No conflict with D-08 (item 4 depends on it but does not contradict it) or with T-01/T-20.
- ADR-0012 (library-allocated opaque buffer, `mkpdf_buffer_free`, no callback now): sound and consistent with D-02, `rules/interoperability.md` item 2 (one `*_free` per object) and T-23/T-24. The per-wrapper table: I found no incorrect statement. Specifically [all A, from language knowledge]: Go: cgo pointer rules (Go memory passed to C may not contain Go pointers; C may not retain Go pointers), `C.GoBytes` taking `C.int`, `runtime.LockOSThread` for thread-local state, `cgo.Handle` for `user_data` in a callback design. Python: `ctypes` default `int` return type, `string_at` copies, `__del__` at shutdown. Node: `napi_create_buffer_copy`, `napi_create_external_buffer`, Electron's V8 memory cage restriction on external buffers. .NET: `nuint` for `size_t`, `SafeHandle.ReleaseHandle`, `DangerousAddRef` for a span's lifetime. Java: `NewByteArray`/`SetByteArrayRegion`, `MemorySegment.reinterpret(size, arena, cleanup)` in the Java 22 FFM API, signed `jint`. C++: RAII and the leak window before construction. Gaps: P2-04 (shape), P2-10, P2-11. Rule 6: the writer must enforce `MAX_OUTPUT_BYTES` incrementally during serialisation, or the `Vec` growth exceeds the limit before the check; the ADR does not say this (note for PDF-001).

### Conditions AC-1 to AC-7 and the ordering DEC-001 → FFI-003 → FFI-001 → PDF-001

- AC-1: measurable (owner decision, `panic` key in both profiles, `compile_error!`, CI check). Add the Security Engineer's review record as named evidence. Consistent with security C-2, which has a later deadline (v0.1.0); the earlier of the two applies.
- AC-2: measurable. Conflicts with the roadmap on whether the ADR is inside FFI-003 (P2-05). Ordering with ADR-0012 is backwards (P2-04).
- AC-3: measurable. The roadmap's FFI-003 row pre-empts it (P2-13). Rule file change needs the owner.
- AC-4: measurable. Should gate FFI-003 as well, because FFI-003 changes the header (P2-13).
- AC-5, AC-6: measurable, owner and Security Engineer inputs named. AC-5 overlaps security C-1 (string limit) with a different deadline (PDF-001 start vs FFI-001 merge); the earlier applies.
- AC-7: now has data: run 45097339864 is red on `miri-ffi`. Not met. Also add "toolchain version printed in each job log", as phase 1 asked, or the MSRV and Miri parts cannot be checked (register R-05).
- Circularity: none found. The chain is linear: DEC-001 → FFI-003 → (GATE-1) → PDF-001 → PDF-002 → FFI-001 → FFI-002. The defect is that it is too linear (P2-05), and that the first link should be the D-08 ADR, not ADR-0012 (P2-04).

### Conflicts with the security review

- SEC-F-04 recommends accepting the C contract for v0.1.0 and deciding checked handles at v0.2.0; D-08 and the architecture review (ARCH-F-01, AC-2) build them before FFI-001. Resolved by the owner's D-08 in favour of the architect's position, but the security review text still says the opposite and must be re-baselined (phase 1 RISK-F-05). Until then a reader finds two gate records that disagree.
- SEC-F-13 says "three" functions bypass `guard`; ARCH-F-09 says four. Four is correct [V] (phase 1 RISK-F-11 found the same).
- Deny count: P2-06.
- Severities: the two reviews agree where they overlap (SEC-F-14 and ARCH-F-04 medium, SEC-F-02 and ARCH-F-07 medium, SEC-F-15 low and ARCH-F-05 medium differ: the architect rates header generation higher because the rule is unmet and a CI diff would fail on day one; I agree with medium because it gates FFI-001 and FFI-003).
- The security review marked ADR-0011 and ADR-0012 as not reviewed (`gate-1-security.md:6`). No Security Engineer review of either exists [V by absence]. AC-1 cannot be met until it does.

## What I did not do

- Run any command (see Limits). Read any CI log. Verify any claim about compiler or language behaviour by execution. Check ISO 32000-1 Annex C. Check the wrapper claims against vendor documentation (I did not use web access for this review; the claims are marked [A]).
- Review the `unsafe` code for soundness. Review the threat model beyond the nine rows cited.
- Edit anything other than this file.

## Proposed register changes (for the phase 1 owner to merge)

New entries:

| Proposed ID | Category | Description | L | I | Owner | Mitigation | Status | Review date | Basis | Cross-reference |
|---|---|---|---|---|---|---|---|---|---|---|
| R-23 | Governance, provenance | GATE-1 artefacts (both reviews, ADR-0011, ADR-0012, phase 1 and 2 files, register) and owner decisions D-08, D-09, FFI-003, DEC-001 exist only in the uncommitted working tree. Remote `main` (`a57d899`) has none of them | H | H | project owner | Commit with DCO sign-off before any condition is treated as met | Open, owner action | At once | V (GitHub contents API 404 and absence of D-08 at `a57d899`) | RISK-F-P2-03 |
| R-24 | Tooling, false assurance | The Risk Reviewer's permitted shell commands are denied in two consecutive sessions. No GATE-1 command result has an independent reproduction | H | M | orchestrator, project owner | Fix the agent permission; have someone with shell store command output under `docs/reviews/` | Open | 2026-11-30 | V | RISK-F-P2-01, RISK-F-11 (phase 1) |
| R-25 | Technical, process | ADR-0012 fixes a pointer-shaped buffer handle before the D-08 ADR decides the handle representation. Accepting it in DEC-001 order risks immediate supersession | M | M | architect | Write the D-08 ADR first, or make ADR-0012 shape-neutral | Open | Before DEC-001 acceptance | V | RISK-F-P2-04, AC-2 |
| R-26 | Schedule | GATE-1 depends on FFI-003 implementation (and a green Miri job), and PDF-001 depends on GATE-1. The writer is blocked by FFI work and a CI fix without a design reason | H | M | orchestrator | Split FFI-003 into ADR (Phase 0) and implementation (Phase 1) | Open | 2026-11-30 | V | RISK-F-P2-05, R-17 |
| R-27 | Governance | Model attribution of the three GATE-1 reviews is self-reported and differs between files (`claude-sonnet-5-5`, `claude-opus-5-5`, `claude-fable-5-1`). Independence-of-perspective claims rest on it | M | L | project owner | Record that model lines are self-reported; decide whether it matters for R-11 | Open | 2026-12-07 | V (file contents), U (actual model) | RISK-F-P2-14, R-11 |

Updates to existing entries:

- R-03: add "ADR-0011 drafted (Proposed). Omits `panic_nounwind` aborts (RISK-F-P2-08). No Security Engineer review yet."
- R-04: add "The architecture review also states CI has never run (RISK-F-P2-02). AC-7 is open against run 45097339864."
- R-16 and R-21: add `docs/architecture.md:54,56`, `docs/api/c-abi.md:69,73`, `docs/testing.md:9,15`, and the architecture review itself to the list of stale documents.
- R-22: note the ARM runner exists and ran (closes the "availability unconfirmed" part of CICD-001).

## Conditions for closing phase 2

1. The owner commits the working tree (R-23).
2. The architect re-baselines ARCH-F-17, ARCH-F-12, AC-7 and `docs/architecture.md:56` against run 45097339864, and fixes RISK-F-P2-07.
3. The D-08 ADR ordering is resolved (RISK-F-P2-04) and the roadmap split is made (RISK-F-P2-05), with the owner's approval.
4. The Security Engineer reviews ADR-0011 and ADR-0012, including RISK-F-P2-08 to P2-11.
5. Someone with shell reproduces the architect's command table and stores the output (RISK-F-P2-01, P2-06).

## Escalations

- To the project owner and the Orchestrator now: RISK-F-P2-03 (the gate record is not in the repository) and RISK-F-P2-01 (the independent reviewer cannot run its checks). Neither is high by the severity scale, but both are process false-assurance patterns.
- To the Solution Architect: RISK-F-P2-04 and P2-05 (ordering), P2-08 to P2-11 (ADR text).
- No sign of copied code was looked for in this phase (phase 1 covered provenance).

## Statement on risk acceptance

I accept no risk. No acceptance record is drafted. Only the project owner can accept a risk, in writing in the repository.
