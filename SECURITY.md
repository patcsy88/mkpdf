# Security policy

## Reporting a vulnerability

Do not open a public issue for a security problem.

**Reporting channel: not yet established.** The project owner must set a private contact (for example a security mailbox or GitHub private vulnerability reporting) before the first public release. This is tracked as OQ-01 in `docs/requirements.md`. Until then, this project makes no support commitment.

When a channel exists, a report should include the version, the platform, a description, a way to reproduce it (a minimal input file if possible) and the impact you expect.

## Target handling times (to be confirmed by the owner)

| Step | Target |
|---|---|
| Acknowledge a report | 3 working days |
| First assessment | 10 working days |
| Fix for a critical or high finding | Before the next release. A production release is blocked while one is open |

## Supported versions

There are no supported releases yet. Before v1.0.0, only the latest pre-release gets fixes.

## Scope

In scope: memory-safety errors, panics that cross the C ABI, denial of service through crafted input (unbounded memory or CPU), incorrect output that causes a security effect, and vulnerable dependencies.

Not yet in scope: features that are not implemented (for example parsing, encryption and signatures).

## Process

Findings are rated critical, high, medium or low. Fixes are prepared privately and released with an advisory. See `rules/security.md` and `docs/security/threat-model.md`.
