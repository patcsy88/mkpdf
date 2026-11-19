use pdfkit_core::{Color, Document, Error, Font, Image, Metadata, PageSize};

const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0xF8, 0xCF, 0xC0, 0xF0,
    0x1F, 0x00, 0x05, 0x00, 0x01, 0xFF, 0x89, 0x99, 0x3D, 0x1D, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
    0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

#[test]
fn shapes_image_and_metadata() {
    let mut doc = Document::new();
    doc.set_metadata(Metadata { title: Some("Smoke".into()), ..Default::default() });
    let page = doc.add_page(PageSize::A4).unwrap();
    page.fill_rect(50.0, 50.0, 200.0, 40.0, Color::rgb(200, 30, 30));
    page.stroke_rect(50.0, 100.0, 200.0, 40.0, Color::BLACK, 1.0);
    page.draw_line(50.0, 150.0, 250.0, 150.0, Color::BLACK, 0.5);
    let img = Image::from_bytes(PNG_1X1.to_vec()).unwrap();
    page.draw_image(50.0, 160.0, 40.0, 40.0, &img);
    let bytes = doc.finish().unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
    assert!(bytes.windows(5).any(|w| w == b"%%EOF"));
}

#[test]
fn rejects_bad_input() {
    assert!(matches!(Font::from_bytes(vec![1, 2, 3]), Err(Error::InvalidFont)));
    assert!(matches!(Image::from_bytes(vec![1, 2, 3]), Err(Error::UnsupportedImageFormat)));
    let mut doc = Document::new();
    assert!(matches!(doc.add_page(PageSize::new(0.0, 10.0)), Err(Error::InvalidPageSize { .. })));
}

#[test]
fn text_with_system_font() {
    let path = "/System/Library/Fonts/Supplemental/Arial.ttf";
    let Ok(data) = std::fs::read(path) else { return };
    let font = Font::from_bytes(data).unwrap();
    let mut doc = Document::new();
    let page = doc.add_page(PageSize::LETTER).unwrap();
    page.draw_text(72.0, 100.0, &font, 18.0, Color::BLACK, "Hello, PDF");
    let bytes = doc.finish().unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
}
