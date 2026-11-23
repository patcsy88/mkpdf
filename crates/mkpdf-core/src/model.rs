use crate::error::{Error, ErrorKind};

/// Largest page dimension accepted, in points. PDF 1.7 Annex C recommends 14 400 units
/// as the implementation limit for user-space dimensions.
pub const MAX_PAGE_DIMENSION_PT: f64 = 14_400.0;

/// Maximum number of pages in one document (resource limit, see threat model T-04).
pub const MAX_PAGES: usize = 100_000;

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
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
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
        doc.set_title("Héllo ✓");
        assert_eq!(doc.title(), Some("Héllo ✓"));
    }
}
