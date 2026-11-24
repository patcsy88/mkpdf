# C ABI specification (API-002)

Status: draft. ABI version 0.1. **Not stable.** The ABI is frozen at v1.0.0 (ADR-0004). Header: `include/mkpdf.h`. Implementation: `crates/mkpdf-ffi`.

## Conventions

- Names start with `mkpdf_`. Types start with `Mkpdf`.
- Functions that can fail return `int32_t` status. `0` means success. Results come back through out-parameters.
- Strings that the caller passes in are UTF-8, given as `(const char *ptr, size_t len)`. No NUL terminator is needed. `ptr` may be `NULL` only when `len` is 0.
- Strings that the library returns (`mkpdf_version`, `mkpdf_last_error_message`) are the exception: they are NUL-terminated, with no length. The error text has any NUL byte in the original message replaced by `?`.
- Sizes are `size_t`, lengths in points are `double`.

## Ownership

| Rule | Detail |
|---|---|
| Library owns objects | Handles come from `*_new` and are owned by the library until the matching `*_free` |
| One free | Free each handle exactly once. `free(NULL)` is a no-op. Any other repeat or foreign pointer is undefined behaviour |
| No interior pointers | The library never returns a pointer into a handle's memory, except as listed below |
| Borrowed static strings | `mkpdf_version()` returns a static string. Do not free it |
| Borrowed error string | `mkpdf_last_error_message()` returns a per-thread string, valid until the next mkpdf call on that thread. Copy it if needed. Do not free it |

## Threading

- A handle may be used by one thread at a time. The caller serialises access.
- Different handles may be used from different threads.
- Error text is per thread. Read it on the thread that made the failing call.

## Status codes

Values never change after release.

| Code | Name | Meaning |
|---|---|---|
| 0 | `MKPDF_OK` | Success |
| 1 | `MKPDF_ERR_NULL_POINTER` | A required pointer was null |
| 2 | `MKPDF_ERR_INVALID_ARGUMENT` | Value outside its domain (for example a NaN or an out-of-range page size) |
| 3 | `MKPDF_ERR_INVALID_UTF8` | String was not valid UTF-8 |
| 4 | `MKPDF_ERR_LIMIT_EXCEEDED` | A documented resource limit was exceeded |
| 100 | `MKPDF_ERR_INTERNAL` | A panic was caught at the boundary. This is a library bug |

A successful call to a function that returns a status clears the thread's error text, and a failing one sets it. The other functions (`mkpdf_abi_version`, `mkpdf_version`, `mkpdf_last_error_message`, `mkpdf_document_free`) neither set nor clear it.

## Panics

No panic crosses the boundary. The status-returning functions run inside the `guard` helper, which catches a panic, records the message and returns `MKPDF_ERR_INTERNAL`. Four functions do not use `guard`, because they have no status to return or cannot fail:

- `mkpdf_abi_version` and `mkpdf_version` return a constant and contain no code that can panic.
- `mkpdf_last_error_message` wraps its body in `catch_unwind` and returns `NULL` if that catches a panic.
- `mkpdf_document_free` wraps the release in `catch_unwind`, returns nothing, and sets no error text. A caught panic is ignored and the memory may leak.

This catching relies on the `panic = "unwind"` setting. It is not yet pinned or checked, and a build with `panic = "abort"` would abort the process instead (ADR-0011, Proposed). Out-of-memory and stack overflow abort the process in either mode. `rules/rust-engineering.md` item 5 says every exported function goes through `guard`, which the code does not do for these four functions (review finding ARCH-F-09).

## Functions (ABI 0.1)

| Function | Purpose |
|---|---|
| `uint32_t mkpdf_abi_version(void)` | `(major << 16) | minor`. Check the major version at load time |
| `const char *mkpdf_version(void)` | Library version, static |
| `const char *mkpdf_last_error_message(void)` | Last error on this thread, or `NULL` |
| `int32_t mkpdf_document_new(MkpdfDocument **out)` | Create an empty document |
| `void mkpdf_document_free(MkpdfDocument *doc)` | Release it |
| `int32_t mkpdf_document_set_title(MkpdfDocument *doc, const char *utf8, size_t len)` | Set the title. See "Title limit" below |
| `int32_t mkpdf_document_add_page(MkpdfDocument *doc, double w, double h, size_t *out_index)` | Append a page. `out_index` may be `NULL` |
| `int32_t mkpdf_document_page_count(const MkpdfDocument *doc, size_t *out_count)` | Number of pages |

## Title limit (`mkpdf_document_set_title`)

The title is at most **4096 bytes** of UTF-8 (`MKPDF_MAX_TITLE_BYTES`, `MAX_TITLE_BYTES` in the core). The limit counts bytes, not characters. A title of exactly 4096 bytes is accepted. The title may contain U+0000 (an interior NUL); the library does not reject it.

Checks run in this order, and the first failure decides the status:

| Order | Condition | Status |
|---|---|---|
| 1 | `doc` is `NULL` | `MKPDF_ERR_NULL_POINTER` |
| 2 | `len == 0` | `MKPDF_OK`. The title becomes empty. `utf8` is not used and may be `NULL` |
| 3 | `utf8` is `NULL` and `len > 0` | `MKPDF_ERR_NULL_POINTER`. This comes before the length checks, so a `NULL` `utf8` with a huge `len` returns this status |
| 4 | `len > isize::MAX` (`PTRDIFF_MAX`) | `MKPDF_ERR_INVALID_ARGUMENT` |
| 5 | `len > 4096` | `MKPDF_ERR_LIMIT_EXCEEDED` |
| 6 | Bytes are not valid UTF-8 | `MKPDF_ERR_INVALID_UTF8` |

Steps 4 and 5 run before any byte at `utf8` is read, so the caller does not need to provide readable memory for an over-long `len`. The error text is a fixed string and never contains the input. On any failure the previous title is kept. For a successful call with `len > 0`, `utf8` must point to `len` initialised bytes in one allocation, and no thread may modify them during the call. Wrappers should reject a longer title before the call, and must not truncate in the middle of a UTF-8 sequence.

## Not yet specified

Serialisation (FFI-001), content, fonts and images. No function returns output yet. ADR-0012 (Proposed, not accepted) proposes a library-allocated buffer, reached through a token and released with `mkpdf_buffer_free`. No callback or caller-supplied buffer is proposed. ADR-0013 (Proposed) changes every handle to a `uint64_t` token. None of this is in the header or the code.

## Header status

`include/mkpdf.h` is **written by hand** for now. `cbindgen` 0.29.4 is installed locally and was run once to compare, but no CI job generates the header and compares it with the committed copy yet (FFI-001, ARCH-F-17 problem 2).
