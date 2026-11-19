use std::sync::Arc;

use crate::{Error, Result};

#[derive(Clone)]
pub struct Font {
    pub(crate) inner: krilla::text::Font,
    data: Arc<Vec<u8>>,
    index: u32,
    units_per_em: f32,
    ascender: f32,
    descender: f32,
    line_gap: f32,
}

impl Font {
    pub fn from_bytes(data: Vec<u8>) -> Result<Self> {
        Self::from_bytes_index(data, 0)
    }

    pub fn from_bytes_index(data: Vec<u8>, index: u32) -> Result<Self> {
        let data = Arc::new(data);
        let inner =
            krilla::text::Font::new(data.clone().into(), index).ok_or(Error::InvalidFont)?;
        let face = rustybuzz::Face::from_slice(&data, index).ok_or(Error::InvalidFont)?;
        let units_per_em = face.units_per_em() as f32;
        if units_per_em <= 0.0 {
            return Err(Error::InvalidFont);
        }
        let (ascender, descender, line_gap) = (
            face.ascender() as f32,
            face.descender() as f32,
            face.line_gap() as f32,
        );
        Ok(Self {
            inner,
            data,
            index,
            units_per_em,
            ascender,
            descender,
            line_gap,
        })
    }

    /// Width of `text` in points at `size`, including kerning.
    pub fn measure(&self, text: &str, size: f32) -> f32 {
        if text.is_empty() {
            return 0.0;
        }
        let face =
            rustybuzz::Face::from_slice(&self.data, self.index).expect("validated in constructor");
        let mut buf = rustybuzz::UnicodeBuffer::new();
        buf.push_str(text);
        let out = rustybuzz::shape(&face, &[], buf);
        let units: i32 = out.glyph_positions().iter().map(|p| p.x_advance).sum();
        units as f32 * size / self.units_per_em
    }

    /// Distance from the baseline to the top of the line, in points.
    pub fn ascent(&self, size: f32) -> f32 {
        self.ascender * size / self.units_per_em
    }

    /// Distance from the baseline to the bottom of the line, in points (positive).
    pub fn descent(&self, size: f32) -> f32 {
        -self.descender * size / self.units_per_em
    }

    pub fn line_gap(&self, size: f32) -> f32 {
        self.line_gap * size / self.units_per_em
    }
}

#[cfg(feature = "bundled-fonts")]
impl Font {
    pub fn roboto_regular() -> Self {
        Self::bundled(include_bytes!("../fonts/Roboto-Regular.ttf"))
    }

    pub fn roboto_bold() -> Self {
        Self::bundled(include_bytes!("../fonts/Roboto-Bold.ttf"))
    }

    pub fn roboto_italic() -> Self {
        Self::bundled(include_bytes!("../fonts/Roboto-Italic.ttf"))
    }

    pub fn roboto_bold_italic() -> Self {
        Self::bundled(include_bytes!("../fonts/Roboto-BoldItalic.ttf"))
    }

    fn bundled(data: &'static [u8]) -> Self {
        Self::from_bytes(data.to_vec()).expect("bundled font is valid")
    }
}
