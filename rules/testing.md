# Testing rules

1. **Unit tests** sit next to the code. Each public function has tests for the normal case, the boundaries and the invalid inputs.
2. **Integration tests** in `tests/` directories exercise the public API only. The FFI crate has tests that call the exported functions the way a C caller would.
3. **Regression tests.** Each fixed bug gets a test that fails without the fix.
4. **Property and fuzz tests.** Every parser and every function that takes untrusted bytes has a fuzz target (`cargo-fuzz`, nightly in CI only). Crashes are kept as regression inputs.
5. **Conformance.** Generated PDFs are checked by external validators, run as black boxes: `qpdf --check` and poppler (`pdfinfo`, `pdftotext`) at minimum. PDF/A and PDF/UA need veraPDF. Record the tool name and version with each result. A test that needs a missing tool must fail in CI. It may be skipped locally, and it must say so in the output.
6. **Determinism.** Tests check that the same input yields byte-identical output.
7. **Memory.** Run Miri on `mkpdf-ffi` tests. Where practical, run sanitizers on the C and C++ binding tests.
8. **Cross-platform.** CI builds and tests on Linux, macOS and Windows.
9. **Flaky tests** are bugs. Do not retry them. Fix or remove them.
10. **Reporting.** A task is not complete without the test command, its result and the date, in the report. "Should work" is not evidence.
11. **Coverage.** Track coverage trends, but do not use a number as a substitute for review.
