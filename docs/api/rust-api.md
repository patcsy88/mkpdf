# Rust API specification (API-001)

Status: draft. Crate `mkpdf-core` 0.0.1. The API is unstable while the version is 0.x. It matches the code at the time of writing. A change to the public items needs an update here.

## Types

### `Document`
Owns pages and metadata.

| Item | Signature | Notes |
|---|---|---|
| `new` | `fn new() -> Document` | Empty. Also `Default` |
| `set_title` | `fn set_title(&mut self, impl AsRef<str>) -> Result<(), Error>` | A UTF-8 string of at most `MAX_TITLE_BYTES` bytes. `LimitExceeded` when longer, and the previous title is kept |
| `title` | `fn title(&self) -> Option<&str>` | |
| `add_page` | `fn add_page(&mut self, Page) -> Result<usize, Error>` | Returns the zero-based index. `LimitExceeded` past `MAX_PAGES` |
| `page_count` | `fn page_count(&self) -> usize` | |
| `pages` | `fn pages(&self) -> &[Page]` | |

### `Page`
| Item | Signature | Notes |
|---|---|---|
| `new` | `fn new(width_pt: f64, height_pt: f64) -> Result<Page, Error>` | Each side finite, greater than 0, at most `MAX_PAGE_DIMENSION_PT`. Otherwise `InvalidArgument` |
| `a4` | `fn a4() -> Page` | 595.276 x 841.89 pt |
| `width_pt`, `height_pt` | `fn(&self) -> f64` | |

### `Error` and `ErrorKind`
`ErrorKind` is `#[non_exhaustive]`: `InvalidArgument`, `LimitExceeded`. `Error::kind()` is the contract. `Error::message()` is for people.

### Constants
`MAX_PAGES` = 100 000. `MAX_TITLE_BYTES` = 4096. `MAX_PAGE_DIMENSION_PT` = 14 400.0. `VERSION`.

## Guarantees

- No panics on invalid input. No `unsafe`. No I/O.
- `Document` contains only owned data (`Option<String>`, `Vec<Page>`), so the compiler makes it `Send` and `Sync`. No test asserts this.

## Derived traits

| Type | Traits |
|---|---|
| `Document` | `Debug`, `Clone`, `Default`, `PartialEq` |
| `Page` | `Debug`, `Clone`, `PartialEq` |
| `Error` | `Debug`, `Clone`, `PartialEq`, `Eq`, plus `Display` and `std::error::Error` |
| `ErrorKind` | `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq` (`#[non_exhaustive]`) |

Adding or removing a derive on a public type is an API change.

## Planned (not implemented)

Serialisation to bytes (PDF-001, PDF-002), content and graphics, fonts and images, layout, a PDF version option (ADR-0006), determinism options (ADR-0007). **No PDF can be produced by this version.**

## Open points

- Whether page size is stored in points only, or in typed units.
- The shape of the serialisation entry point (return `Vec<u8>`, or write to a sink).
