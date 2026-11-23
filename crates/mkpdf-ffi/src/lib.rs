//! Stable C ABI for mkpdf. The contract is in `docs/api/c-abi.md` and `include/mkpdf.h`.
//!
//! Rules enforced here:
//! - every exported function runs inside [`guard`], so no panic crosses the boundary;
//! - every pointer argument is null-checked before use;
//! - the library owns every object it hands out; callers release them with the matching
//!   `*_free` function exactly once.

#![allow(unsafe_code)]
#![allow(clippy::missing_safety_doc)]

use std::cell::RefCell;
use std::ffi::{CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};

use mkpdf_core::{Document, Error, ErrorKind, Page};

/// ABI major version. Incremented on any breaking change.
pub const ABI_MAJOR: u32 = 0;
/// ABI minor version. Incremented on backwards-compatible additions.
pub const ABI_MINOR: u32 = 1;

/// Status codes. Values are part of the ABI and never change once released.
pub const MKPDF_OK: i32 = 0;
/// A required pointer argument was null.
pub const MKPDF_ERR_NULL_POINTER: i32 = 1;
/// An argument was outside its documented domain.
pub const MKPDF_ERR_INVALID_ARGUMENT: i32 = 2;
/// A string argument was not valid UTF-8.
pub const MKPDF_ERR_INVALID_UTF8: i32 = 3;
/// A documented resource limit was exceeded.
pub const MKPDF_ERR_LIMIT_EXCEEDED: i32 = 4;
/// An internal panic was caught at the boundary. Indicates a bug.
pub const MKPDF_ERR_INTERNAL: i32 = 100;

/// Opaque document handle.
pub struct MkpdfDocument {
    inner: Document,
}

enum Failure {
    NullPointer(&'static str),
    InvalidUtf8,
    Core(Error),
}

impl Failure {
    fn code(&self) -> i32 {
        match self {
            Failure::NullPointer(_) => MKPDF_ERR_NULL_POINTER,
            Failure::InvalidUtf8 => MKPDF_ERR_INVALID_UTF8,
            Failure::Core(e) => match e.kind() {
                ErrorKind::LimitExceeded => MKPDF_ERR_LIMIT_EXCEEDED,
                _ => MKPDF_ERR_INVALID_ARGUMENT,
            },
        }
    }

    fn message(&self) -> String {
        match self {
            Failure::NullPointer(name) => format!("argument `{name}` is null"),
            Failure::InvalidUtf8 => "string argument is not valid UTF-8".to_owned(),
            Failure::Core(e) => e.message().to_owned(),
        }
    }
}

impl From<Error> for Failure {
    fn from(e: Error) -> Self {
        Failure::Core(e)
    }
}

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

fn set_last_error(message: &str) {
    let sanitized = message.replace('\0', "?");
    // `try_with` and `try_borrow_mut` avoid panicking during thread teardown.
    let _ = LAST_ERROR.try_with(|slot| {
        if let Ok(mut slot) = slot.try_borrow_mut() {
            *slot = CString::new(sanitized).ok();
        }
    });
}

fn clear_last_error() {
    let _ = LAST_ERROR.try_with(|slot| {
        if let Ok(mut slot) = slot.try_borrow_mut() {
            *slot = None;
        }
    });
}

fn guard(f: impl FnOnce() -> Result<(), Failure>) -> i32 {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(())) => {
            clear_last_error();
            MKPDF_OK
        }
        Ok(Err(failure)) => {
            set_last_error(&failure.message());
            failure.code()
        }
        Err(_) => {
            set_last_error("internal error: panic caught at FFI boundary");
            MKPDF_ERR_INTERNAL
        }
    }
}

/// Returns the ABI version as `(major << 16) | minor`.
#[unsafe(no_mangle)]
pub extern "C" fn mkpdf_abi_version() -> u32 {
    (ABI_MAJOR << 16) | ABI_MINOR
}

/// Returns the library version as a static NUL-terminated string. Never freed by the caller.
#[unsafe(no_mangle)]
pub extern "C" fn mkpdf_version() -> *const c_char {
    static VERSION: &std::ffi::CStr = match std::ffi::CStr::from_bytes_with_nul(
        concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes(),
    ) {
        Ok(s) => s,
        Err(_) => c"unknown",
    };
    VERSION.as_ptr()
}

/// Returns the last error message of the calling thread, or null if the previous call
/// succeeded. The pointer is valid until the next mkpdf call on the same thread.
#[unsafe(no_mangle)]
pub extern "C" fn mkpdf_last_error_message() -> *const c_char {
    catch_unwind(|| {
        LAST_ERROR
            .try_with(|slot| {
                slot.try_borrow()
                    .ok()
                    .and_then(|s| s.as_ref().map(|c| c.as_ptr()))
                    .unwrap_or(std::ptr::null())
            })
            .unwrap_or(std::ptr::null())
    })
    .unwrap_or(std::ptr::null())
}

/// Creates an empty document. On success `*out` receives an owned handle.
///
/// # Safety
/// `out` must be null or point to writable memory for one pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mkpdf_document_new(out: *mut *mut MkpdfDocument) -> i32 {
    guard(|| {
        if out.is_null() {
            return Err(Failure::NullPointer("out"));
        }
        let handle = Box::into_raw(Box::new(MkpdfDocument {
            inner: Document::new(),
        }));
        // SAFETY: `out` is non-null and the caller guarantees it is writable.
        unsafe { *out = handle };
        Ok(())
    })
}

/// Releases a document. Passing null is a no-op. Passing any other pointer more than once,
/// or a pointer not returned by this library, is undefined behaviour.
///
/// # Safety
/// `doc` must be null or a live handle from `mkpdf_document_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mkpdf_document_free(doc: *mut MkpdfDocument) {
    if doc.is_null() {
        return;
    }
    // A panic in Drop must not cross the boundary; ignore the result.
    let _ = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: caller contract guarantees `doc` came from `Box::into_raw` and is unique.
        drop(unsafe { Box::from_raw(doc) });
    }));
}

/// Sets the document title from `len` bytes of UTF-8 at `utf8`. No NUL terminator required.
///
/// # Safety
/// `doc` must be a live handle. `utf8` must be readable for `len` bytes (it may be null only
/// when `len` is 0).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mkpdf_document_set_title(
    doc: *mut MkpdfDocument,
    utf8: *const c_char,
    len: usize,
) -> i32 {
    guard(|| {
        // SAFETY: non-null checked by `as_mut`; caller guarantees liveness and exclusivity.
        let doc = unsafe { doc.as_mut() }.ok_or(Failure::NullPointer("doc"))?;
        let bytes: &[u8] = if len == 0 {
            &[]
        } else if utf8.is_null() {
            return Err(Failure::NullPointer("utf8"));
        } else {
            // SAFETY: non-null and caller guarantees `len` readable bytes.
            unsafe { std::slice::from_raw_parts(utf8.cast::<u8>(), len) }
        };
        let title = std::str::from_utf8(bytes).map_err(|_| Failure::InvalidUtf8)?;
        doc.inner.set_title(title);
        Ok(())
    })
}

/// Appends a page of `width_pt` x `height_pt` points. If `out_index` is non-null it receives
/// the zero-based page index.
///
/// # Safety
/// `doc` must be a live handle. `out_index` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mkpdf_document_add_page(
    doc: *mut MkpdfDocument,
    width_pt: f64,
    height_pt: f64,
    out_index: *mut usize,
) -> i32 {
    guard(|| {
        // SAFETY: see `mkpdf_document_set_title`.
        let doc = unsafe { doc.as_mut() }.ok_or(Failure::NullPointer("doc"))?;
        let index = doc.inner.add_page(Page::new(width_pt, height_pt)?)?;
        if !out_index.is_null() {
            // SAFETY: non-null and caller guarantees it is writable.
            unsafe { *out_index = index };
        }
        Ok(())
    })
}

/// Writes the number of pages to `*out_count`.
///
/// # Safety
/// `doc` must be a live handle and `out_count` writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mkpdf_document_page_count(
    doc: *const MkpdfDocument,
    out_count: *mut usize,
) -> i32 {
    guard(|| {
        // SAFETY: non-null checked by `as_ref`; caller guarantees liveness.
        let doc = unsafe { doc.as_ref() }.ok_or(Failure::NullPointer("doc"))?;
        if out_count.is_null() {
            return Err(Failure::NullPointer("out_count"));
        }
        // SAFETY: non-null and caller guarantees it is writable.
        unsafe { *out_count = doc.inner.page_count() };
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    use std::ptr;

    fn last_error() -> Option<String> {
        let p = mkpdf_last_error_message();
        // SAFETY: pointer is either null or a valid NUL-terminated string from this library.
        (!p.is_null()).then(|| unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned())
    }

    #[test]
    fn abi_version_packs_major_minor() {
        assert_eq!(mkpdf_abi_version(), (ABI_MAJOR << 16) | ABI_MINOR);
    }

    #[test]
    fn version_is_nonempty_c_string() {
        // SAFETY: static NUL-terminated string.
        let v = unsafe { CStr::from_ptr(mkpdf_version()) };
        assert_eq!(v.to_str().unwrap(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn lifecycle_and_pages() {
        let mut doc = ptr::null_mut();
        unsafe {
            assert_eq!(mkpdf_document_new(&raw mut doc), MKPDF_OK);
            assert!(!doc.is_null());
            let title = "Titel ✓";
            assert_eq!(
                mkpdf_document_set_title(doc, title.as_ptr().cast(), title.len()),
                MKPDF_OK
            );
            let mut idx = usize::MAX;
            assert_eq!(
                mkpdf_document_add_page(doc, 595.0, 842.0, &raw mut idx),
                MKPDF_OK
            );
            assert_eq!(idx, 0);
            assert_eq!(
                mkpdf_document_add_page(doc, 100.0, 100.0, ptr::null_mut()),
                MKPDF_OK
            );
            let mut n = 0usize;
            assert_eq!(mkpdf_document_page_count(doc, &raw mut n), MKPDF_OK);
            assert_eq!(n, 2);
            assert_eq!((*doc).inner.title(), Some("Titel ✓"));
            mkpdf_document_free(doc);
            mkpdf_document_free(ptr::null_mut());
        }
    }

    #[test]
    fn null_pointers_are_rejected() {
        unsafe {
            assert_eq!(mkpdf_document_new(ptr::null_mut()), MKPDF_ERR_NULL_POINTER);
            assert!(last_error().unwrap().contains("out"));
            assert_eq!(
                mkpdf_document_add_page(ptr::null_mut(), 1.0, 1.0, ptr::null_mut()),
                MKPDF_ERR_NULL_POINTER
            );
            assert_eq!(
                mkpdf_document_page_count(ptr::null(), ptr::null_mut()),
                MKPDF_ERR_NULL_POINTER
            );
        }
    }

    #[test]
    fn invalid_inputs_set_error_and_success_clears_it() {
        let mut doc = ptr::null_mut();
        unsafe {
            assert_eq!(mkpdf_document_new(&raw mut doc), MKPDF_OK);
            assert_eq!(
                mkpdf_document_add_page(doc, f64::NAN, 10.0, ptr::null_mut()),
                MKPDF_ERR_INVALID_ARGUMENT
            );
            assert!(last_error().is_some());
            let bad = [0xff_u8, 0xfe];
            assert_eq!(
                mkpdf_document_set_title(doc, bad.as_ptr().cast(), bad.len()),
                MKPDF_ERR_INVALID_UTF8
            );
            assert_eq!(
                mkpdf_document_set_title(doc, ptr::null(), 3),
                MKPDF_ERR_NULL_POINTER
            );
            assert_eq!(mkpdf_document_set_title(doc, ptr::null(), 0), MKPDF_OK);
            assert!(last_error().is_none());
            mkpdf_document_free(doc);
        }
    }

    #[test]
    fn panic_is_contained() {
        let code = guard(|| panic!("boom"));
        assert_eq!(code, MKPDF_ERR_INTERNAL);
        assert!(last_error().unwrap().contains("panic"));
    }
}
