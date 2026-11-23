# AGENTS.md: mkpdf engineering instructions

mkpdf is an open-source (Apache-2.0), Rust-based PDF generation and processing engine. Every language binding consumes the same Rust core through a stable C ABI. No SDK may duplicate PDF logic.

## Mandatory reading

All agents MUST follow these files. If two of them conflict, stop and escalate to the Solution Architect.

- `rules/architecture.md`
- `rules/rust-engineering.md`
- `rules/pdf-standards.md`
- `rules/interoperability.md`
- `rules/security.md`
- `rules/testing.md`
- `rules/licensing.md`

Context: `docs/requirements.md` (decisions and open questions), `docs/roadmap.md` (releases and backlog), `docs/architecture.md`, `docs/adr/`.

## Repository map

| Path | Purpose |
|---|---|
| `crates/mkpdf-core` | Document model and PDF engine. Safe Rust only, no I/O |
| `crates/mkpdf-ffi` | Stable C ABI. The only crate that may contain `unsafe` |
| `crates/mkpdf-cli` | `mkpdf` command line |
| `include/` | C header |
| `rules/` | Binding rules |
| `docs/` | Architecture, ADRs, API specs, threat model, roadmap |
| `.opencode/agents/` | Agent definitions |
| `.github/workflows/` | CI |

## Commands

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
```

## Agents

Defined in `.opencode/agents/`. Ten roles: the nine required by the project brief (`orchestrator`, `architect`, `pdf-standards`, `rust-core`, `interop`, `qa`, `security`, `devops`, `docs`) plus `risk-reviewer`, added by the project owner on 2026-11-23 (OQ-14). The Risk Reviewer is an independent second reviewer that challenges other agents' evidence and keeps the risk register in `docs/risk/`. Each file states responsibilities, scope, permissions, deliverables, quality requirements and escalation rules.

## Gates

A task is not done until the relevant gates pass, with evidence:

1. Formatting and linting
2. Unit and integration tests
3. Security checks, with review by the Security Engineer when the rules require it
4. Dependency licence check (`cargo deny`)
5. Cross-platform build (CI)
6. Regression tests
7. PDF validation by a named external validator, when PDF output changes
8. Documentation review

The Orchestrator coordinates the work but cannot waive the architecture, security or QA gate. Only the project owner can accept a risk, and the acceptance is recorded in the repository.

## Reporting format

Report every completed task with: objective, approach, files changed, tests executed, test results, security implications, architectural implications, outstanding issues and the next recommended action. Do not claim completion without evidence. Say which checks were not run, and why.

## Ambiguity

If a requirement is ambiguous, state the ambiguity and propose alternatives before an irreversible decision, and record the answer in `docs/requirements.md`.
