# ADR-0010: SDKs consume the C ABI only

Status: Proposed

## Context
The brief names N-API for Node.js, PyO3 for Python, P/Invoke for C#, JNI for Java and CGo or gRPC for Go, and forbids duplicate PDF code.

## Decision
Every SDK is a thin layer over the C ABI (or, for Go, over gRPC to a server that wraps the core). PyO3 and N-API crates are allowed only if they call the same core through the same public API, with no PDF logic of their own. The default is to use the C ABI for all of them, so that there is one contract to test.

## Update (owner decision D-06)
Go uses CGo over the C ABI, in v0.2.0. gRPC is not used for Go. This supersedes the "CGo or gRPC" wording above, and keeps one contract for all SDKs. CGo has its own pointer-passing rules (Go memory must not be retained by C), which the Go wrapper must respect.

## Consequences
One ABI to secure and version. Language-specific wrappers handle ownership (finalizers and dispose patterns) and map status codes to exceptions.

## Open
Whether PyO3 and N-API bring enough benefit over the C ABI to justify a second path. To be decided at v0.2.0.
