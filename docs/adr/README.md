# Architecture decision records

An ADR records one significant decision: context, decision, alternatives and consequences. Accepted ADRs are not edited, other than their status. Supersede them with a new ADR.

Statuses: Proposed, Accepted, Superseded, Rejected. All ADRs below are **Proposed** until GATE-1 reviews them, except where the owner's decision is quoted.

| ID | Title | Status |
|---|---|---|
| [0001](0001-record-decisions.md) | Record architecture decisions | Proposed |
| [0002](0002-workspace-and-unsafe-boundary.md) | Cargo workspace and the unsafe boundary | Proposed |
| [0003](0003-opaque-handle-c-abi.md) | Opaque-handle C ABI | Proposed (owner decision D-02) |
| [0004](0004-abi-and-api-versioning.md) | ABI and API versioning | Proposed |
| [0005](0005-dependency-policy.md) | Dependency allowlist policy | Proposed (owner decision D-03) |
| [0006](0006-pdf-version-strategy.md) | PDF 1.7 and 2.0 strategy | Proposed |
| [0007](0007-determinism.md) | Deterministic output | Proposed (updated for owner decision D-12: deterministic by default, timestamps and IDs opt-in) |
| [0008](0008-error-model.md) | Error model | Proposed |
| [0009](0009-wasm-and-no-io-core.md) | No-I/O core and WebAssembly | Proposed |
| [0010](0010-sdks-consume-c-abi.md) | SDKs consume the C ABI only | Proposed |
| [0011](0011-panic-strategy.md) | Panic strategy (`unwind` or `abort`), resolves OQ-12 | Proposed (recommends `unwind`, pinned and enforced. Amended 2026-11-24 for SEC-ADR-06 to 09 and RISK-F-P2-02, 08, 09) |
| [0012](0012-output-buffer-ownership.md) | Output-buffer ownership across the C ABI, resolves OQ-13 | Proposed (library-allocated buffer, token handle, depends on ADR-0013. Amended 2026-11-24 for SEC-ADR-10 to 13, 15 and RISK-F-P2-04, 10, 11. `MAX_OUTPUT_BYTES` not decided) |
| [0013](0013-generation-checked-handles.md) | Generation-checked handles (owner decisions D-08, D-11; backlog FFI-003), amends ADR-0003 | Proposed (D-13 to D-18 and D-20 recorded as decided. Amended 2026-11-24 for SEC-ADR-01 to 05, 14, 15. Only acceptance and the byte values are NOT DECIDED) |

ADR-0013 must be accepted before FFI-001 starts. A resource-limits ADR is listed in `docs/requirements.md` ("Architectural decisions identified") and does not exist.
