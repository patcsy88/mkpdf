use std::sync::Arc;

use crate::{Error, Result};

#[derive(Clone)]
pub struct Font {
    pub(crate) inner: krilla::text::Font,
    data: Arc<Vec<u8>>,
    index: u32,
}

impl Font {
    pub fn from_bytes(data: Vec<u8>) -> Result<Self> {
        Self::from_bytes_index(data, 0)
    }

    pub fn from_bytes_index(data: Vec<u8>, index: u32) -> Result<Self> {
        let data = Arc::new(data);
        let inner = krilla::text::Font::new(data.clone().into(), index).ok_or(Error::InvalidFont)?;
        rustybuzz::Face::from_slice(&data, index).ok_or(Error::InvalidFont)?;
        Ok(Self { inner, data, index })
    }

    fn face(&self) -> rustybuzz::Face<'_> {
        rustybuzz::Face::from_slice(&self.data, self.index).expect("validated in constructor")
    }

    /// Width of `text` in points at `size`, including kerning.
    pub fn measure(&self, text: &str, size: f32) -> f32 {
        if text.is_empty() {
            return 0.0;
        }
        let face = self.face();
        let mut buf = rustybuzz::UnicodeBuffer::new();
        buf.push_str(text);
        let out = rustybuzz::shape(&face, &[], buf);
        let units: i32 = out.glyph_positions().iter().map(|p| p.x_advance).sum();
        units as f32 * size / face.units_per_em() as f32
    }

    /// Distance from the baseline to the top of the line, in points.
    pub fn ascent(&self, size: f32) -> f32 {
        let face = self.face();
        face.ascender() as f32 * size / face.units_per_em() as f32
    }

    /// Distance from the baseline to the bottom of the line, in points (positive).
    pub fn descent(&self, size: f32) -> f32 {
        let face = self.face();
        -(face.descender() as f32) * size / face.units_per_em() as f32
    }

    pub fn line_gap(&self, size: f32) -> f32 {
        let face = self.face();
        face.line_gap() as f32 * size / face.units_per_em() as f32
    }
}
