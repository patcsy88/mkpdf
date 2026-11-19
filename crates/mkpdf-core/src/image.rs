use std::sync::Arc;

use crate::{Error, Result};

#[derive(Clone)]
pub struct Image {
    pub(crate) inner: krilla::image::Image,
}

impl Image {
    pub fn from_bytes(data: Vec<u8>) -> Result<Self> {
        let kind = sniff(&data).ok_or(Error::UnsupportedImageFormat)?;
        let data: krilla::Data = Arc::new(data).into();
        let inner = match kind {
            Kind::Png => krilla::image::Image::from_png(data, true),
            Kind::Jpeg => krilla::image::Image::from_jpeg(data, true),
            Kind::Gif => krilla::image::Image::from_gif(data, true),
            Kind::Webp => krilla::image::Image::from_webp(data, true),
        }
        .map_err(Error::InvalidImage)?;
        Ok(Self { inner })
    }

    pub fn size(&self) -> (u32, u32) {
        self.inner.size()
    }
}

enum Kind {
    Png,
    Jpeg,
    Gif,
    Webp,
}

fn sniff(d: &[u8]) -> Option<Kind> {
    if d.starts_with(&[0x89, b'P', b'N', b'G']) {
        Some(Kind::Png)
    } else if d.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some(Kind::Jpeg)
    } else if d.starts_with(b"GIF8") {
        Some(Kind::Gif)
    } else if d.len() > 12 && &d[0..4] == b"RIFF" && &d[8..12] == b"WEBP" {
        Some(Kind::Webp)
    } else {
        None
    }
}
