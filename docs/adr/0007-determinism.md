# ADR-0007: Deterministic output

Status: Proposed. Open question OQ-10.

## Decision
Same document, same options, same bytes. The core does not read the clock or a random source. A creation date and a document ID are written only if the caller supplies them. Collections that affect output use ordered structures. A test checks byte equality across two runs.

## Consequences
Reproducible tests and builds. Callers who want a timestamp pass it in. Whether the default document ID is derived from the content (a hash) or left out is to be decided with the writer.
