# Security rules

1. **Threat model.** `docs/security/threat-model.md` is the reference. A change that adds an input path, an FFI function, a dependency, a network feature or a cryptographic feature must update it.
2. **Hostile input.** Treat PDFs, fonts, images and strings from callers as hostile. Bound sizes, nesting depth, object counts and decompressed sizes. Reject, do not guess.
3. **Memory safety.** See `rules/rust-engineering.md`. Run Miri on the FFI crate. Fuzz every parser.
4. **No network.** The core never opens network connections and never loads external resources by itself. Offline operation is the default.
5. **Filesystem.** The core does not read or write files. File access happens in the CLI or in the caller.
6. **Cryptography.** Never implement primitives ourselves. Use reviewed crates chosen through an ADR. Encryption and signature features need a security review before release.
7. **Secrets.** No keys, tokens or personal data in the repository, in logs or in test fixtures.
8. **Dependencies.** Run `cargo deny check` and `cargo audit` in CI. A new dependency needs a licence and security review (`rules/licensing.md`).
9. **Severity policy.** A finding is critical, high, medium or low. No unresolved critical or high finding may be accepted for a production release. Medium and low findings need a recorded owner and date.
10. **Disclosure.** Reports go through `SECURITY.md`. Fixes are prepared privately and released with an advisory.
11. **Review.** The Security Engineer must approve changes to `mkpdf-ffi`, to anything that parses input, and to any cryptographic code.
