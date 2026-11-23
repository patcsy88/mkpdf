# Architecture rules

Binding on all agents and contributors.

1. **One engine.** All PDF logic lives in Rust under `crates/`. SDKs and bindings must not implement PDF logic. They only marshal data to and from the C ABI (or another interface that wraps the core).
2. **Layering.** `mkpdf-core` has no I/O, no global state, no threads and no platform dependency, so it compiles to WebAssembly. `mkpdf-ffi` is the only crate that may contain `unsafe`. `mkpdf-cli` and future servers depend on the core, never the reverse.
3. **Public surface is explicit.** Everything public in `mkpdf-core` is documented, and a change to it needs an API review. Use `#[non_exhaustive]` on public enums and structs that may grow.
4. **Versioning.** The Rust API follows SemVer. The C ABI has its own major and minor version (`mkpdf_abi_version`). A breaking ABI change needs an ADR and a major bump.
5. **ADRs.** A significant decision (a new dependency, a public API shape, a security trade-off, a new crate) needs an ADR in `docs/adr/` before it is merged. ADRs are immutable once accepted. Supersede them with a new one.
6. **Resource limits.** Every input-driven allocation, recursion and loop needs a documented limit. Limits are constants in the core and are covered by tests.
7. **Determinism.** Output for the same input and options must be byte-identical unless an option says otherwise. Do not read the clock or random sources inside the core without an explicit option.
8. **No unnecessary abstraction.** Do not add traits, frameworks or crates for a single use.
9. **Gates.** The Orchestrator cannot waive the architecture gate (GATE-1 and each release gate), the security gate or the QA gate.
