# ADR-0006: PDF 1.7 and 2.0 strategy

Status: Proposed

## Context
The brief requires PDF 1.7 compatibility and that PDF 2.0 support be addressed in the architecture. The standards are ISO 32000-1 and ISO 32000-2.

## Decision
- Output defaults to PDF 1.7.
- The document carries a target version. The writer emits only features that the target version permits, and the header (and the catalog `Version` entry where needed) matches it.
- PDF 2.0 features (for example some encryption and signature features) are available only when the caller selects 2.0.
- The parser (v0.4.0) accepts both versions and tolerates real-world deviations, within limits, but never reports a file as conformant on that basis.
- Conformance claims need an external validator (`rules/pdf-standards.md`).

## Consequences
The version is a field of the model from the start. The ISO documents are not freely available. The team uses the public material available and records its sources.

## Open
Which validator covers PDF 2.0 output (OQ-11).
