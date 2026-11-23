use crate::error::{Error, ErrorKind};

/// Largest page dimension accepted, in points. PDF 1.7 Annex C recommends 14 400 units
/// as the implementation limit for user-space dimensions.
pub const MAX_PAGE_DIMENSION_PT: f64 = 14_400.0;

/// Maximum number of pages in one document (resource limit, see threat model T-04).
pub const MAX_PAGES: usize = 100_000;

/// Maximum length of the document title, in bytes of UTF-8 (resource limit, see
/// threat model T-10).
pub const MAX_TITLE_BYTES: usize = 4096;

/// A single page. Content is added in later milestones.
#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    width_pt: f64,
    height_pt: f64,
}

impl Page {
    /// Create a page of the given size in points (1/72 inch).
    ///
    /// Both dimensions must be finite, greater than zero and at most
    /// [`MAX_PAGE_DIMENSION_PT`].
    pub fn new(width_pt: f64, height_pt: f64) -> Result<Self, Error> {
        for (name, v) in [("width", width_pt), ("height", height_pt)] {
            if !v.is_finite() || v <= 0.0 || v > MAX_PAGE_DIMENSION_PT {
                return Err(Error::new(
                    ErrorKind::InvalidArgument,
                    format!("page {name} must be finite and in (0, {MAX_PAGE_DIMENSION_PT}]"),
                ));
            }
        }
        Ok(Self {
            width_pt,
            height_pt,
        })
    }

    /// ISO 216 A4 portrait.
    #[must_use]
    pub fn a4() -> Self {
        Self {
            width_pt: 595.276,
            height_pt: 841.89,
        }
    }

    /// Width in points.
    #[must_use]
    pub fn width_pt(&self) -> f64 {
        self.width_pt
    }

    /// Height in points.
    #[must_use]
    pub fn height_pt(&self) -> f64 {
        self.height_pt
    }
}

/// An in-memory document. Owns all of its pages and metadata.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Document {
    title: Option<String>,
    pages: Vec<Page>,
}

impl Document {
    /// Create an empty document.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the document title metadata.
    ///
    /// The title is measured in bytes of UTF-8, not in characters, and may be at most
    /// [`MAX_TITLE_BYTES`] long. The length is checked before the input is copied.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorKind::LimitExceeded`] when the title is longer than
    /// [`MAX_TITLE_BYTES`]. The previous title is left unchanged. The error message
    /// does not contain the input.
    pub fn set_title(&mut self, title: impl AsRef<str>) -> Result<(), Error> {
        let title = title.as_ref();
        if title.len() > MAX_TITLE_BYTES {
            return Err(Error::new(
                ErrorKind::LimitExceeded,
                format!("title exceeds {MAX_TITLE_BYTES} bytes"),
            ));
        }
        self.title = Some(title.to_owned());
        Ok(())
    }

    /// Document title, if set.
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Append a page and return its zero-based index.
    pub fn add_page(&mut self, page: Page) -> Result<usize, Error> {
        if self.pages.len() >= MAX_PAGES {
            return Err(Error::new(
                ErrorKind::LimitExceeded,
                format!("document exceeds {MAX_PAGES} pages"),
            ));
        }
        self.pages.push(page);
        Ok(self.pages.len() - 1)
    }

    /// Number of pages.
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Pages in order.
    #[must_use]
    pub fn pages(&self) -> &[Page] {
        &self.pages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_rejects_invalid_sizes() {
        for (w, h) in [
            (0.0, 10.0),
            (10.0, -1.0),
            (f64::NAN, 10.0),
            (10.0, f64::INFINITY),
            (MAX_PAGE_DIMENSION_PT + 1.0, 10.0),
        ] {
            let err = Page::new(w, h).unwrap_err();
            assert_eq!(err.kind(), ErrorKind::InvalidArgument, "{w} x {h}");
        }
    }

    #[test]
    fn page_accepts_boundary() {
        assert!(Page::new(MAX_PAGE_DIMENSION_PT, 0.001).is_ok());
    }

    #[test]
    fn document_tracks_pages_in_order() {
        let mut doc = Document::new();
        assert_eq!(doc.add_page(Page::a4()).unwrap(), 0);
        assert_eq!(doc.add_page(Page::new(100.0, 200.0).unwrap()).unwrap(), 1);
        assert_eq!(doc.page_count(), 2);
        assert!((doc.pages()[1].height_pt() - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn document_title_roundtrip() {
        let mut doc = Document::new();
        assert_eq!(doc.title(), None);
        doc.set_title("Héllo ✓").unwrap();
        assert_eq!(doc.title(), Some("Héllo ✓"));
    }

    #[test]
    fn title_at_limit_is_accepted() {
        let mut doc = Document::new();
        let title = "a".repeat(MAX_TITLE_BYTES);
        doc.set_title(&title).unwrap();
        assert_eq!(doc.title(), Some(title.as_str()));
    }

    #[test]
    fn title_over_limit_is_rejected_and_old_title_kept() {
        let mut doc = Document::new();
        doc.set_title("old").unwrap();
        let err = doc.set_title("a".repeat(MAX_TITLE_BYTES + 1)).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::LimitExceeded);
        assert!(!err.message().contains("aaaa"));
        assert_eq!(doc.title(), Some("old"));
    }

    #[test]
    fn title_limit_counts_bytes_not_chars() {
        let mut doc = Document::new();
        // "é" is 2 bytes, so this title has exactly MAX_TITLE_BYTES bytes.
        let fits = format!("{}é", "a".repeat(MAX_TITLE_BYTES - 2));
        assert_eq!(fits.len(), MAX_TITLE_BYTES);
        doc.set_title(&fits).unwrap();
        // The 2-byte char starts at the last allowed byte and ends past it. Its char
        // count is within the limit, so only a byte count rejects it.
        let straddles = format!("{}é", "a".repeat(MAX_TITLE_BYTES - 1));
        assert_eq!(straddles.len(), MAX_TITLE_BYTES + 1);
        assert!(straddles.chars().count() <= MAX_TITLE_BYTES);
        let err = doc.set_title(&straddles).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::LimitExceeded);
        assert_eq!(doc.title(), Some(fits.as_str()));
    }

    #[test]
    fn max_pages_is_enforced() {
        let mut doc = Document::new();
        let page = Page::a4();
        for _ in 0..MAX_PAGES {
            doc.add_page(page.clone()).unwrap();
        }
        assert_eq!(doc.page_count(), MAX_PAGES);
        let err = doc.add_page(page).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::LimitExceeded);
        assert_eq!(doc.page_count(), MAX_PAGES);
    }
}
