use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid font data")]
    InvalidFont,
    #[error("invalid image: {0}")]
    InvalidImage(String),
    #[error("unsupported image format")]
    UnsupportedImageFormat,
    #[error("invalid page size {width}x{height}")]
    InvalidPageSize { width: f32, height: f32 },
    #[error("invalid geometry")]
    InvalidGeometry,
    #[error("pdf generation failed: {0}")]
    Pdf(String),
}
