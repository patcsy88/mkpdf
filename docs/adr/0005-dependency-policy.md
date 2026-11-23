# ADR-0005: Dependency allowlist policy

Status: Proposed. Follows owner decision D-03.

## Context
Writing everything in-house (compression, font parsing, shaping, cryptography) raises the risk of defects and the cost. Unreviewed dependencies raise supply-chain and licence risk.

## Decision
- The core has no dependencies today.
- A small set of permissively licensed crates may be added, each after review (`rules/licensing.md`) and with its own ADR for significant ones. Candidates, not yet chosen (OQ-05): a Deflate implementation, font parsing, text shaping, hashing and cryptography.
- Licences are limited to the list in `deny.toml`. `cargo deny` and `cargo audit` run in CI.
- Cryptographic primitives are never written by us.
- Dependencies are recorded in `docs/dependencies.md`.

## Alternatives
Zero runtime dependencies: maximum control, but much slower and riskier for compression, fonts and cryptography.

## Consequences
Each addition has a cost in review. The allowlist keeps the supply chain short.
