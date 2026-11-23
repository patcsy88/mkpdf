# GATE-1 phase 1: independent risk review

- Reviewer: Risk Reviewer (agent role, claude-sonnet-5-5). Second review of `docs/reviews/gate-1-security.md`. I did not write that file.
- Date: 2026-11-23
- Subject: the security review (written against `59df15e`), CI run 45097339864 (head `a57d899`), and the overstated-wording check on `docs/requirements.md`, `docs/roadmap.md`, `README.md`, `CHANGELOG.md`.
- Out of scope: `docs/reviews/gate-1-architecture.md`, ADR-0011, ADR-0012. A follow-up covers them. I did not open the architecture review.
- Marking: every statement is tagged **[V]** verified (I saw the file, line or API output), **[U]** unverified, or **[A]** assumed.

## Recommendation

**PASS WITH CONDITIONS** for phase 1, and **not** a clean close of GATE-1.

- No critical finding. One high finding (RISK-F-07: no human review of AI-generated unsafe code and of the reviews themselves). 
- The security review is thorough on code reading and honest about what it did not run. Its main weaknesses are that it is now stale against CI, that it is the same model reviewing its own project's output, and that a few claims are unsupported.
- C-3 of the security review ("green CI run on all jobs") is **not met**: `miri-ffi` failed. [V]
- Nothing here accepts a risk. Only the project owner can.
- **Limits of this review:** shell was denied in this session, even for the commands my role allows (`gh run`, `git log`, `cargo test`). Job logs return HTTP 403 without authentication. I could not read any CI log, run any test, or run `git log`. Everything about CI below comes from public job metadata. Findings that depend on logs are marked [U] and need a re-check by someone with `gh` access.

## What was examined

- [V] `AGENTS.md`, all of `rules/`, `.opencode/agents/risk-reviewer.md`, `docs/requirements.md`, `docs/roadmap.md`, `docs/security/threat-model.md`, `docs/reviews/gate-1-security.md` (all 239 lines).
- [V] `crates/mkpdf-ffi/src/lib.rs`, `crates/mkpdf-core/src/{lib,error,model}.rs`, `crates/mkpdf-cli/src/main.rs`, `include/mkpdf.h`, `deny.toml`, `Cargo.toml`, `rust-toolchain.toml`, `clippy.toml`, `.github/workflows/ci.yml` (the only file under `.github/`).
- [V] `README.md`, `CHANGELOG.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `NOTICE`, `CONTRIBUTING.md` (wording grep), `docs/api/{c-abi,rust-api}.md`, `docs/architecture.md`, `docs/testing.md`, `docs/dependencies.md`, ADR-0003 and the ADR status lines.
- [V] Public GitHub API output: workflow run list, job list for run 45097339864, private vulnerability reporting status, repository owner type, commit metadata for `59df15e` and the compare to `a57d899`.

## What was not examined

- [U] CI job logs (HTTP 403). So: the cause of the Miri failure, the `rustc --version` of each job, test counts per OS.
- [U] Branch protection (HTTP 401), secret scanning, Dependabot, required reviews, repository rulesets.
- [U] I ran no command. I did not re-run clippy, tests, deny, nm or strings. The security review's local results (clippy, 10 tests, deny, `nm` showing 8 symbols, no home path in the dylib) are **[U]** for me. They are plausible and consistent with the code I read (10 `#[test]` attributes: 4 in core, 6 in ffi, **[V]** by grep).
- [U] Whether `a57d899` differs from `59df15e` beyond `Cargo.toml` repository metadata and `SECURITY.md`. The compare output was truncated. 
- Not examined by design: architecture review, ADR-0011, ADR-0012, ADRs 0001-0002 and 0004-0010 beyond status lines.
- No technical re-review of `unsafe`. That belongs to the Security Engineer.

## CI run 45097339864 (head `a57d899`, branch main, push, 2026-11-23)

| Job | Conclusion | What it actually shows |
|---|---|---|
| lint | success [V] | fmt and clippy passed on the current stable. Plausible. Log not read [U] |
| test (ubuntu-latest, ubuntu-24.04-arm, macos-latest, windows-latest) | success [V] | `cargo test --workspace` took 0 to 1 s per job. Consistent with 10 small tests, test counts not visible [U]. Also shows `ubuntu-24.04-arm` exists for this repository [V] |
| msrv | success [V] | Ran about 8 s with `dtolnay/rust-toolchain@1.85` then `cargo check --workspace`. No step prints the compiler version [V]. Whether it used 1.85 or stable via `rust-toolchain.toml` is **unverified** (RISK-F-02) |
| wasm-core | success [V] | Built `mkpdf-core` for `wasm32-unknown-unknown`. Shows a build only, nothing was run |
| deny | success [V] | Nothing to check: no third-party dependency (RISK-F-03) |
| audit | success [V] | Installed cargo-audit (3 m 12 s), ran 2 s. Nothing to scan (RISK-F-03) |
| miri-ffi | **failure** [V] | Toolchain step passed, then `cargo miri test -p mkpdf-ffi` failed in about 0 s, annotation "exit code 1". Cause [U]. A missing Miri component under the stable override fits SEC-F-06 but I did not see it |

## Findings

| ID | Severity | Title | Disposition |
|---|---|---|---|
| RISK-F-01 | medium | Miri job failed, condition C-3 and SEC-F-05 are open, and the review text does not know | Fix, keep green log, re-baseline review |
| RISK-F-02 | medium | The MSRV job may not test 1.85 and still shows green | Print `rustc --version`, force toolchain |
| RISK-F-03 | low | `deny` and `audit` pass because there is nothing to check | Label, re-review at first dependency |
| RISK-F-04 | low | Green `test` and `lint` jobs are unproven beyond conclusion | Print counts and versions |
| RISK-F-05 | medium | The security review is stale: C-3, C-5, SEC-F-04, SEC-F-09, C-6 | Revise the review against `a57d899` |
| RISK-F-06 | low | Conduct alias and "when a channel exists" wording remain | Owner action, then wording fix |
| RISK-F-07 | high | AI-generated code and review with no human review | Owner appoints a human reviewer |
| RISK-F-08 | low | Documents contradict each other on tool state and review state | Docs fix |
| RISK-F-09 | low | Owner commits are unsigned and use a local email | Owner decision |
| RISK-F-10 | medium | Clean-room claim in `NOTICE` has no evidence | Evidence or softer wording |
| RISK-F-11 | low | Unsupported or unchecked claims inside the security review | Correct in next revision |
| RISK-F-12 | low | Conditions C-1 to C-6 are partly vague or unverifiable | Reword with evidence tests |

### RISK-F-01 (medium): Miri failed
- Evidence: job `miri-ffi` conclusion failure, step "Run cargo miri test -p mkpdf-ffi" failed in about 0 s [V, API job list]. `gate-1-security.md:37` says "CI has never run" and C-3 (line 228) needs a green run on all jobs [V]. Cause not visible [U].
- Assessment: the job failing is the honest outcome and not false assurance. The risk is that SEC-F-05 and SEC-F-06 are now partly confirmed in effect (probably), but the review still says "unverified", and no doc records the failure. The unsafe code has still had **no** dynamic check [V by absence].
- Disposition: devops and qa read the log, fix the toolchain selection (`cargo +nightly miri`, or `RUSTUP_TOOLCHAIN`), and keep a green log in the repository. Do not close SEC-F-05 without it. Re-check: someone with `gh run view 45097339864 --log-failed`.

### RISK-F-02 (medium): MSRV job may be a false pass
- Evidence: `ci.yml` msrv job uses `dtolnay/rust-toolchain@1.85` then `cargo check --workspace`, no version print [V]. `rust-toolchain.toml` has `channel = "stable"` [V]. Job passed [V]. Whether the override applies is the SEC-F-06 prediction, which I judge likely [A]: rustup uses a `rust-toolchain.toml` in the working directory over the default.
- Impact: any statement that MSRV 1.85 is verified (OQ-04, `Cargo.toml` `rust-version`) would rest on a job that may have used stable. If so it is the clearest false-assurance case in this run.
- Disposition: until a log shows `rustc 1.85.x`, treat MSRV as unverified. Wording in docs must say so.

### RISK-F-03 (low): vacuous passes
- Evidence: `Cargo.lock` lists only the three workspace packages [V]. The security review says the same (`gate-1-security.md:31`) [V]. `audit` ran 2 s [V].
- Impact: green `deny` and `audit` jobs show the plumbing works and nothing else. `docs/dependencies.md` is honest about this. A reader of a badge would not be.
- Disposition: note it where the CI status is reported. Re-run this review at the first dependency (ADR-0005).

### RISK-F-04 (low): other green jobs not proven
- Evidence: `test` jobs ran 0 to 1 s. 10 tests exist [V by grep]. No log read [U]. A test job that matched zero tests would also pass.
- Disposition: devops adds `rustc --version` and `cargo test` output to a kept artefact, or a step that asserts the minimum test count.

### RISK-F-05 (medium): the security review is stale
- Evidence: reviewed commit `59df15e` (`gate-1-security.md:5`). CI ran on `a57d899`, a later commit [V].
  - `gate-1-security.md:37` "CI has never run": no longer true. [V]
  - SEC-F-09 and C-6 say there is no reporting channel. GitHub API returned `{"enabled":true}` for private vulnerability reporting, and `SECURITY.md` line 7 says so [V]. The conduct alias remains open (RISK-F-06).
  - SEC-F-04 and C-5 say owner decision is pending and recommend option A for v0.1.0. `docs/requirements.md` D-08 now amends ADR-0003 toward generation-checked handles [V from earlier read; I did not re-open D-08 in this pass]. The review does not mention D-08.
  - The review lists the ARM Linux job as unrun. It ran [V].
- Impact: the conditions list is the main gate artefact. A reader would act on out-of-date items.
- Disposition: Security Engineer revises against the current commit and the CI run. Mark findings as changed, not silently edited.

### RISK-F-06 (low): remaining wording
- Evidence: `SECURITY.md` line 9 still says "When a channel exists" and handling times are "to be confirmed" [V from earlier read]. `CODE_OF_CONDUCT.md` line 4 contains the placeholder pointing at OQ-01 [V]. OQ-01 conduct alias is deferred by the owner.
- Disposition: owner action before first release. Docs wording updated when the alias exists.

### RISK-F-07 (high): no human review
- Evidence: `docs/dependencies.md` states all code is AI-assisted with no independent human review [V from earlier read]. `gate-1-security.md:3` says it is a single-reviewer review by an agent [V]. This review is by an agent of the same model family [V]. `rules/rust-engineering.md` item 3 requires a second reviewer for each new unsafe function. `rules/licensing.md` item 5 requires equal review of AI-generated content [V from earlier read]. No human review of any `unsafe` block, ADR, or review exists in the repository [V by absence in what I read].
- Impact: independent in role is not independent in perspective. Shared blind spots between agents are likely [A]. It is also the base of the provenance claim (RISK-F-10).
- Disposition: escalate to the project owner and the Orchestrator now. Appoint a human reviewer for `crates/mkpdf-ffi/**`, for each ADR before acceptance, and for the licence and provenance statements. Record who reviewed what.

### RISK-F-08 (low): internal contradictions
All [V] unless marked.
- `docs/testing.md` line 11: "qpdf is not installed here". `docs/roadmap.md` line 97 and `docs/dependencies.md` line 29: qpdf 12.4.2 installed.
- `docs/api/c-abi.md` line 66: "cbindgen is not installed here". `docs/roadmap.md` line 96 and `docs/dependencies.md` line 28: installed, 0.29.4.
- `docs/dependencies.md` line 26: cargo-deny "not run yet on the workspace", while the security review says it ran locally.
- `docs/architecture.md` line 72: "No architecture or security review has been done". A security review exists.
- `docs/roadmap.md` CICD-001 "Draft (unrun)" (line 53) and "availability unconfirmed": CI ran, including the ARM runner. ARCH-003 "Done (macOS only)", `requirements.md` OQ-03 and OQ-04 "never run" wording: stale.
- Checked and **not** a problem: `AGENTS.md` line 43, `requirements.md` D-09 and `roadmap.md` AGT-001 already say ten agents. OQ-14 is not stale on that point.
- README: states no PDF can be produced and makes no conformance claim [V]. `CONTRIBUTING.md` line 33 forbids compliance claims without a named validator [V]. The README SDK list omits Go while D-06 adds it (minor) [V from earlier read]. I found no "stable", "validated" or "passes" wording that overstates what exists, apart from the stale statuses above. `mkpdf.h` calls the ABI "unstable until v1.0.0" [V from earlier read], and "stable C ABI" in `AGENTS.md` lines 3 and 24 is a design aim, not a current property [A]. I did not read `CHANGELOG.md` line by line for a second time in this pass beyond the grep results.
- Disposition: docs fixes by the Docs agent. Do not mark any of these as validated.

### RISK-F-09 (low): unsigned commits
- Evidence: commit API shows `verification.verified: false, reason: unsigned` for `59df15e` [V]. The owner's commits use `patcsy88@MBPM2.local` [V]. They carry DCO sign-offs [V]. Only the GitHub-generated initial commit is signed [V].
- Disposition: owner decides on signing before the first release. Tag and artefact signing is already a release-gate item (SEC-F-07 and C-4).

### RISK-F-10 (medium): clean-room claim
- Evidence: `NOTICE` lines 6-8 say mkpdf is "an independent, clean-room implementation" developed from public standards, with no code copied or adapted from other PDF libraries [V]. No provenance record, no list of sources consulted, no similarity check exists (glob for a provenance file found none) [V]. The code is AI-generated (RISK-F-07). Today the code is a small skeleton, so a copy is unlikely [A].
- Impact: a legal statement that nobody can currently support. "Clean room" normally means a separation of people who read the other code from people who write.
- Disposition: either remove "clean-room" and keep "developed from public specifications", or add a provenance record before release. Owner decides, because it is a legal statement.

### RISK-F-11 (low): unsupported claims inside the security review
- "6 unsafe in non-test code" (`gate-1-security.md:20`): my grep of `lib.rs` found `unsafe` blocks at lines 162, 180, 197, 204, 226, 230, 247, 252, which is eight, not six [V for the lines, count of "blocks" depends on definition]. Re-count and list them.
- The line-72 statement that standard-library debug assertions would abort a debug build for an oversized `len` is stated as fact. I did not verify it [A].
- Local results (clippy, tests, deny, `nm`, `strings`) cannot be re-checked from the document [U]. Not wrong, but no command output is stored in the repository.
- "No problem found by reading" for aliasing and provenance (SEC-F-05) is a reading result, which the review itself says. Fine.
- SEC-F-01 core claim (`from_raw_parts` with oversized `len` is UB, `lib.rs:204`) is correct [V].
- SEC-F-13 is correct: four exported functions do not use `guard` [V]. The review says "three". The text lists four. Count mismatch.
- SEC-F-16 claim that CI does not use `--locked` is correct [V].
- Rating: I agree with the severity of all findings I could check. SEC-F-04 at medium is defensible only while no stale-handle detection exists and the owner has decided D-08. I would not raise it.

### RISK-F-12 (low): conditions C-1 to C-6
- C-1: clear (commit, tests, review). Test names should be required.
- C-2: "ADR-0011 accepted after security review". Out of scope here, only the owner can accept.
- C-3: "green CI run on all jobs": not met. Add "with a printed toolchain version per job" or the Miri and MSRV items cannot be checked.
- C-4: "Pin all actions and tools", plus "repository settings" as evidence. Settings cannot be seen from the repository. Ask for a settings export or screenshot with date.
- C-5: superseded by D-08 (see RISK-F-05).
- C-6: partly met (private reporting on). Split into reporting channel (done) and conduct alias (open).
- Vague terms: "before FFI-001 merges" and "before first release" need a named milestone commit. Proposed dates are not agreed. The review says the Orchestrator must confirm them [V].

## Risk register
`docs/risk/register.md` holds 22 entries. Coverage: technical, security, supply chain, provenance and clean room, licensing, standards claims, tooling, schedule, owner dependency (conduct alias), the organisation move, and AI-generated code with no human review. No entry is accepted.

## Escalations
- To the project owner and Orchestrator now: RISK-F-07 (high). Also RISK-F-10 (legal wording).
- Possible false assurance: RISK-F-02 (MSRV job), RISK-F-03 (deny and audit).
- No sign of copied code was found. I did not run a similarity check [U].

## Conditions for closing phase 1
1. Someone with `gh` access reads the `miri-ffi` and `msrv` logs and records the result (RISK-F-01, RISK-F-02).
2. The Security Engineer re-baselines the review against `a57d899` and D-08 (RISK-F-05, RISK-F-11).
3. The owner records a decision on human review (RISK-F-07) and on the clean-room wording (RISK-F-10).
4. The Docs agent fixes the contradictions in RISK-F-08.

## Statement on risk acceptance
I accept no risk. No acceptance record is drafted. Only the project owner can accept a risk, in writing in the repository.
