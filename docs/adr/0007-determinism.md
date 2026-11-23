# ADR-0007: Deterministic output

Status: Proposed. Updated on 2026-11-23 to record owner decision D-12 (resolves open question OQ-10). Not accepted. Only the project owner accepts it.

## What changed in this revision

- The earlier text left open whether the default document ID is derived from the content (a hash) or left out. Owner decision D-12 settles the default: **deterministic by default, no timestamp and no random document ID. Timestamps and IDs are opt-in.** This revision states that.
- The "to be decided with the writer" sentence in Consequences is removed.
- Added: a list of what the default output contains and omits, what the caller can switch on, and the points that D-12 does not settle.
- The status stays Proposed.

## Decision

Same document, same options, same bytes. This holds across runs, machines and platforms for the same library version.

1. **Default.** With default options the output has no creation date, no modification date, no random value and no document ID. The core does not read the clock or a random source, and no hidden source of variation (hash-map iteration order, addresses, thread timing) reaches the output.
2. **Opt-in.** A creation date, a modification date and a document ID are written only when the caller supplies them as explicit options. The core writes exactly the value it is given. It never makes one up, and it never calls the clock. Callers who want a timestamp pass it in.
3. **Document ID.** The default is no `/ID` entry. A caller may supply one. Deriving an ID from a hash of the content is not part of the default. Whether it may be offered later as an explicit option (for example "derive the ID from the content") is not decided here, and it would need the hash dependency to pass ADR-0005.
4. **Ordering.** Collections that affect output use ordered structures. Object numbers and the order of objects are a function of the model only.
5. **Test.** A test checks byte equality across two runs of the same input in one process, and a golden file or hash check across builds guards against drift. The tests that read files supply fixed dates and IDs explicitly.

## Not settled by D-12

- **Standards that need an ID or a date.** Some profiles may require a trailer `/ID` (PDF/A is the likely case, and encryption needs one). If so, a request for that profile without a caller-supplied ID has to return an error or require an ID option, and not invent one. This is from the architect's recollection and has not been checked against the standards. The PDF Standards Engineer must confirm it before the writer ships (PDF-001, `rules/pdf-standards.md` item 8).
- **Producer string.** Whether the output names the library and version (`/Producer`). The same library version still gives the same bytes, so it does not break the rule. Different versions differ. It is also a small privacy and fingerprinting point (threat T-15). Not decided here.
- **Metadata values from the caller** (title, author) are deterministic by nature and are not affected.

## Alternatives

- **Derive the ID from a content hash by default.** Still deterministic. Rejected as the default by D-12, and it needs a hash dependency before the first line of the writer exists. It may return as an opt-in (item 3).
- **Always write the current time and a random ID** (what many PDF libraries do). Not reproducible, makes tests and builds depend on the clock, and writes information the caller did not ask for. Rejected.
- **Make the clock available to the core behind a trait.** The core does no I/O and has no platform dependency (ADR-0009). The caller passes the value in, so no trait is needed. Rejected.

## Consequences

- Reproducible tests, golden files and reproducible builds without special flags.
- A caller that wants a creation date or an ID must pass it in. SDK wrappers expose these as explicit optional parameters, and none sets them by default. A wrapper must not fill them from its own clock.
- Output made without the options may lack metadata that readers show, such as a creation date. This is by design.
- The API shape of the opt-in options is not defined yet. It belongs with the writer (PDF-001) and the options type in `docs/api/rust-api.md`.
- ADR-0006 (the PDF version as a model field) and the writer must write the trailer without `/ID` and without `/Info` dates when the options are absent, and a test must cover it.
