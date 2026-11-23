# Contributing to mkpdf

Thank you for helping. Please read `AGENTS.md` and the files in `rules/` first. They apply to people and to AI agents.

## Ground rules

- **Clean room.** Write from public standards. Do not copy or adapt code from other PDF libraries, and do not read their source while implementing the same feature. If you already have, say so in the pull request.
- **DCO.** Sign off each commit with `git commit -s`. This certifies the [Developer Certificate of Origin](https://developercertificate.org/) 1.1.
- **Licence.** Contributions are made under Apache-2.0.
- **Security issues** go to the process in `SECURITY.md`, not to public issues.
- **Conduct.** See `CODE_OF_CONDUCT.md`.

## Workflow

1. Open an issue for anything beyond a small fix. A new dependency, public API or ABI change needs an ADR (`docs/adr/`).
2. Create a branch, make focused commits and add tests.
3. Run the gates:

   ```
   cargo fmt --all --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cargo deny check
   ```

4. Open a pull request. State what you tested, what you did not test and what could go wrong. Changes to `mkpdf-ffi`, to parsers or to cryptography need a Security Engineer review.

## Code standards

- Safe Rust by default. `unsafe` only in `mkpdf-ffi`, with a `// SAFETY:` comment (see `rules/rust-engineering.md`).
- No `unwrap`, `expect` or `panic!` in library code.
- Public items are documented.
- No claim of standards compliance without a named validator run.

## AI-assisted contributions

You are responsible for AI-generated code. It gets the same review, and the same provenance rules, as any other code.
