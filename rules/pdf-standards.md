# PDF standards rules

1. **References.** Use only public standards: ISO 32000-1 (PDF 1.7), ISO 32000-2 (PDF 2.0), and the referenced specifications (OpenType, Unicode, ICC, and others). Cite the clause in a code comment or ADR when implementing a rule.
2. **Clean room.** Do not read, copy or adapt the source code of other PDF libraries. Do not paste output from them as a reference implementation. Validators may be run as black boxes on our output. See `rules/licensing.md`.
3. **Version strategy.** The writer targets PDF 1.7 by default. PDF 2.0 features are opt-in, selected by an explicit version option, and the header and catalog version must be consistent with the features used. See ADR-0006.
4. **Output.** Output must be well formed: correct header, binary marker comment, cross-reference data with correct offsets, trailer and `startxref`. Never emit a feature that the declared version does not allow.
5. **Input.** A parser is hostile-input code. It must have size, depth and object-count limits, must tolerate no input as trusted, and must have fuzz targets before it is released.
6. **Claims.** No document, README, release note or API doc may say "PDF/A compliant", "PDF/UA compliant", "valid PDF 2.0" or similar unless a named validator and its version were run in CI on representative output. State the validator in the claim. Until then use "intended to conform" or say nothing.
7. **Fonts and text.** Embed fonts only if their licence allows embedding. Text is stored as Unicode with a proper `ToUnicode` map so that extraction works.
8. **Deterministic output.** No timestamps or random IDs unless the caller asks for them.
