# C ABI specification (API-002)

Status: draft. ABI version 0.1. **Not stable.** The ABI is frozen at v1.0.0 (ADR-0004). Header: `include/mkpdf.h`. Implementation: `crates/mkpdf-ffi`.

## Conventions

- Names start with `mkpdf_`. Types start with `Mkpdf`.
- Functions that can fail return `int32_t` status. `0` means success. Results come back through out-parameters.
- Strings are UTF-8, given as `(const char *ptr, size_t len)`. No NUL terminator is needed. `ptr` may be `NULL` only when `len` is 0.
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

A successful call clears the thread's error text. A failing call sets it.

## Panics

No panic crosses the boundary. Each exported function catches it, records the message and returns `MKPDF_ERR_INTERNAL`. `mkpdf_document_free` also catches panics and returns nothing.

## Functions (ABI 0.1)

| Function | Purpose |
|---|---|
| `uint32_t mkpdf_abi_version(void)` | `(major << 16) | minor`. Check the major version at load time |
| `const char *mkpdf_version(void)` | Library version, static |
| `const char *mkpdf_last_error_message(void)` | Last error on this thread, or `NULL` |
| `int32_t mkpdf_document_new(MkpdfDocument **out)` | Create an empty document |
| `void mkpdf_document_free(MkpdfDocument *doc)` | Release it |
| `int32_t mkpdf_document_set_title(MkpdfDocument *doc, const char *utf8, size_t len)` | Set the title |
| `int32_t mkpdf_document_add_page(MkpdfDocument *doc, double w, double h, size_t *out_index)` | Append a page. `out_index` may be `NULL` |
| `int32_t mkpdf_document_page_count(const MkpdfDocument *doc, size_t *out_count)` | Number of pages |

## Not yet specified

Serialisation to a caller buffer or callback (FFI-001), content, fonts and images. The output-buffer ownership model (library-allocated buffer with `mkpdf_buffer_free`, or caller-supplied) must be settled before FFI-001, and written as an ADR.

## Header status

`include/mkpdf.h` is **written by hand** for now, because `cbindgen` is not installed here. CI must generate the header and compare it with the committed copy (FFI-001).
