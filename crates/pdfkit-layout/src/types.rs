use pdfkit_core::{Color, Font, Image, PageSize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] pdfkit_core::Error),
    #[error("table row {row} has {got} cells but the table has {expected} columns")]
    ColumnMismatch { row: usize, got: usize, expected: usize },
    #[error("table has no columns")]
    EmptyTable,
    #[error("page content area is too small for the given margins")]
    NoContentArea,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Margins {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Margins {
    pub const fn all(v: f32) -> Self {
        Self { top: v, right: v, bottom: v, left: v }
    }
}

#[derive(Clone)]
pub struct TextStyle {
    pub font: Font,
    pub size: f32,
    pub color: Color,
    /// Multiplier of `size`, e.g. 1.3.
    pub line_height: f32,
}

impl TextStyle {
    pub fn new(font: Font, size: f32) -> Self {
        Self { font, size, color: Color::BLACK, line_height: 1.3 }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn line_height(mut self, v: f32) -> Self {
        self.line_height = v;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Column {
    /// Fixed width in points.
    Fixed(f32),
    /// Share of the width left after fixed columns, relative to other weights.
    Weight(f32),
}

#[derive(Clone)]
pub struct Cell {
    pub text: String,
    pub style: Option<TextStyle>,
    pub align: Align,
    pub background: Option<Color>,
}

impl Cell {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into(), style: None, align: Align::Left, background: None }
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = Some(style);
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }
}

pub type Row = Vec<Cell>;

#[derive(Clone)]
pub struct Table {
    pub columns: Vec<Column>,
    /// Repeated at the top of every page the table continues onto.
    pub header: Option<Row>,
    pub rows: Vec<Row>,
    pub style: TextStyle,
    pub padding: f32,
    pub border: Option<(Color, f32)>,
}

impl Table {
    pub fn new(columns: Vec<Column>, style: TextStyle) -> Self {
        Self {
            columns,
            header: None,
            rows: Vec::new(),
            style,
            padding: 4.0,
            border: Some((Color::rgb(160, 160, 160), 0.5)),
        }
    }
}

#[derive(Clone)]
pub enum Block {
    Paragraph { text: String, style: TextStyle, align: Align, space_after: f32 },
    Table(Table),
    /// `width` in points; defaults to the image's pixel width, capped to the content width.
    Image { image: Image, width: Option<f32>, align: Align, space_after: f32 },
    Spacer(f32),
    PageBreak,
}

impl Block {
    pub fn paragraph(text: impl Into<String>, style: TextStyle) -> Self {
        Block::Paragraph { text: text.into(), style, align: Align::Left, space_after: 6.0 }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PageInfo {
    /// 1-based.
    pub number: usize,
    pub total: usize,
    pub size: PageSize,
}
