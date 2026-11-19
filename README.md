# mkpdf

A Rust library for creating PDFs, intended to be callable from Rust, .NET, Java, Python, Node.js and Go.

> Status: early development. Only the Rust core and layout crates exist so far.

## Crates

| Crate | Purpose |
|---|---|
| `mkpdf-core` | Document model: pages, text with embedded custom fonts, shapes, images (PNG, JPEG, GIF, WebP), metadata. Built on [`krilla`](https://crates.io/crates/krilla). |
| `mkpdf-layout` | Flow layout: word-wrapped paragraphs, tables with repeating headers, images, automatic page breaks, headers and footers with page numbers. |

## Planned

- Reading, editing, merging and splitting existing PDFs
- C ABI and bindings for Python, Node.js, .NET, Java and Go
- HTML/CSS subset to PDF
- Invoice and report templates

## Licensing

mkpdf is licensed under the Apache License 2.0 (see `LICENSE-APACHE` and `NOTICE`).

The optional `bundled-fonts` feature of `mkpdf-core` embeds the Roboto fonts (Apache-2.0, Google). Its license text is in `crates/mkpdf-core/fonts/LICENSE-Roboto`.

All dependencies must use permissive licenses (MIT, Apache-2.0, BSD, ISC, Zlib, Unicode and similar). This is enforced with [`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny):

```
cargo deny check licenses sources
```

## Development

```
cargo build
cargo test
```
