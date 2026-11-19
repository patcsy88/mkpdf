use krilla::geom::{PathBuilder, Point, Rect, Size};
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{Fill, Stroke};
use krilla::text::TextDirection;

use crate::{Color, Error, Font, Image, Result};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageSize {
    pub width: f32,
    pub height: f32,
}

impl PageSize {
    pub const A4: PageSize = PageSize { width: 595.28, height: 841.89 };
    pub const LETTER: PageSize = PageSize { width: 612.0, height: 792.0 };

    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn landscape(self) -> Self {
        Self { width: self.height, height: self.width }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub language: Option<String>,
    pub creator: Option<String>,
}

enum Op {
    Text { x: f32, y: f32, font: Font, size: f32, color: Color, text: String },
    Rect { x: f32, y: f32, w: f32, h: f32, fill: Option<Color>, stroke: Option<(Color, f32)> },
    Line { x1: f32, y1: f32, x2: f32, y2: f32, color: Color, width: f32 },
    Image { x: f32, y: f32, w: f32, h: f32, image: Image },
}

/// A page is a list of drawing operations. The origin is the top-left corner and
/// units are PDF points (1/72 inch).
pub struct Page {
    size: PageSize,
    ops: Vec<Op>,
}

impl Page {
    pub fn size(&self) -> PageSize {
        self.size
    }

    /// `y` is the text baseline.
    pub fn draw_text(&mut self, x: f32, y: f32, font: &Font, size: f32, color: Color, text: &str) {
        self.ops.push(Op::Text { x, y, font: font.clone(), size, color, text: text.to_owned() });
    }

    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        self.ops.push(Op::Rect { x, y, w, h, fill: Some(color), stroke: None });
    }

    pub fn stroke_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color, width: f32) {
        self.ops.push(Op::Rect { x, y, w, h, fill: None, stroke: Some((color, width)) });
    }

    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: Color, width: f32) {
        self.ops.push(Op::Line { x1, y1, x2, y2, color, width });
    }

    pub fn draw_image(&mut self, x: f32, y: f32, w: f32, h: f32, image: &Image) {
        self.ops.push(Op::Image { x, y, w, h, image: image.clone() });
    }
}

#[derive(Default)]
pub struct Document {
    metadata: Metadata,
    pages: Vec<Page>,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_metadata(&mut self, metadata: Metadata) {
        self.metadata = metadata;
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn add_page(&mut self, size: PageSize) -> Result<&mut Page> {
        if !(size.width > 0.0 && size.height > 0.0 && size.width.is_finite() && size.height.is_finite()) {
            return Err(Error::InvalidPageSize { width: size.width, height: size.height });
        }
        self.pages.push(Page { size, ops: Vec::new() });
        Ok(self.pages.last_mut().expect("just pushed"))
    }

    pub fn page_mut(&mut self, index: usize) -> Option<&mut Page> {
        self.pages.get_mut(index)
    }

    pub fn finish(self) -> Result<Vec<u8>> {
        let mut doc = krilla::Document::new();
        doc.set_metadata(build_metadata(&self.metadata));

        for page in &self.pages {
            let settings = PageSettings::from_wh(page.size.width, page.size.height)
                .ok_or(Error::InvalidPageSize { width: page.size.width, height: page.size.height })?;
            let mut kpage = doc.start_page_with(settings);
            let mut surface = kpage.surface();
            for op in &page.ops {
                render_op(&mut surface, op)?;
            }
            surface.finish();
            kpage.finish();
        }

        doc.finish().map_err(|e| Error::Pdf(format!("{e:?}")))
    }
}

fn build_metadata(m: &Metadata) -> krilla::metadata::Metadata {
    let mut out = krilla::metadata::Metadata::new();
    if let Some(v) = &m.title {
        out = out.title(v.clone());
    }
    if !m.authors.is_empty() {
        out = out.authors(m.authors.clone());
    }
    if let Some(v) = &m.description {
        out = out.description(v.clone());
    }
    if !m.keywords.is_empty() {
        out = out.keywords(m.keywords.clone());
    }
    if let Some(v) = &m.language {
        out = out.language(v.clone());
    }
    if let Some(v) = &m.creator {
        out = out.creator(v.clone());
    }
    out
}

fn fill(color: Color) -> Fill {
    Fill { paint: color.to_krilla().into(), opacity: NormalizedF32::ONE, rule: Default::default() }
}

fn stroke(color: Color, width: f32) -> Stroke {
    Stroke { paint: color.to_krilla().into(), width, ..Default::default() }
}

fn render_op(s: &mut krilla::surface::Surface<'_>, op: &Op) -> Result<()> {
    match op {
        Op::Text { x, y, font, size, color, text } => {
            s.set_fill(Some(fill(*color)));
            s.set_stroke(None);
            s.draw_text(Point::from_xy(*x, *y), font.inner.clone(), *size, text, false, TextDirection::Auto);
        }
        Op::Rect { x, y, w, h, fill: f, stroke: st } => {
            let rect = Rect::from_xywh(*x, *y, *w, *h).ok_or(Error::InvalidGeometry)?;
            let mut pb = PathBuilder::new();
            pb.push_rect(rect);
            let path = pb.finish().ok_or(Error::InvalidGeometry)?;
            s.set_fill(f.map(fill));
            s.set_stroke(st.map(|(c, w)| stroke(c, w)));
            s.draw_path(&path);
        }
        Op::Line { x1, y1, x2, y2, color, width } => {
            let mut pb = PathBuilder::new();
            pb.move_to(*x1, *y1);
            pb.line_to(*x2, *y2);
            let path = pb.finish().ok_or(Error::InvalidGeometry)?;
            s.set_fill(None);
            s.set_stroke(Some(stroke(*color, *width)));
            s.draw_path(&path);
        }
        Op::Image { x, y, w, h, image } => {
            let size = Size::from_wh(*w, *h).ok_or(Error::InvalidGeometry)?;
            s.push_transform(&krilla::geom::Transform::from_translate(*x, *y));
            s.draw_image(image.inner.clone(), size);
            s.pop();
        }
    }
    Ok(())
}
