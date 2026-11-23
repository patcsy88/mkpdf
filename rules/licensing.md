# Licensing and provenance rules

1. **Project licence.** Apache-2.0. Every crate sets `license = "Apache-2.0"`. New source files do not need a header, because the repository `LICENSE` applies.
2. **Contributions.** DCO sign-off (`git commit -s`). Contributors confirm that they wrote the code or have the right to submit it.
3. **Clean room.** Implement from public standards. Do not copy or adapt code from other PDF libraries, including from the output of AI tools that may reproduce it. Do not read the source code of other PDF implementations while writing equivalent features. If you have, say so in the pull request so that a reviewer can decide.
4. **Provenance.** Each non-trivial module records its sources (standard clauses, papers) in doc comments or in `docs/provenance.md`.
5. **AI-generated code** gets the same review as any other code. The author is responsible for its provenance.
6. **Dependencies.** A new dependency needs: a name and version, the licence, the maintainers and activity, the transitive tree, a security check (`cargo audit`, advisory history), the reason it is needed and the alternative of not using it, and an entry in `docs/dependencies.md`. Significant additions need an ADR.
7. **Allowed licences** are listed in `deny.toml`: Apache-2.0, MIT, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, Unicode-3.0 and Apache-2.0 WITH LLVM-exception. Copyleft licences (GPL, LGPL, AGPL, MPL, EPL) and unknown licences are not allowed in anything that is linked into the shipped libraries. Build and test tools run as separate processes may be copyleft, but must be recorded.
8. **No services.** The core must not need a commercial licence, subscription, cloud service or external API.
9. **Fonts and test files.** Fonts, images and PDFs in the repository need a recorded licence that allows redistribution.
10. **NOTICE.** Keep it correct when something that needs attribution is added.
