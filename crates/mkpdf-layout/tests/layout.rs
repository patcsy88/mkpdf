use mkpdf_core::{Color, Document, Font, PageSize};
use mkpdf_layout::*;

fn font() -> Font {
    Font::roboto_regular()
}

#[test]
fn long_text_and_table_paginate_with_footer() {
    let font = font();
    let body = TextStyle::new(font.clone(), 11.0);
    let mut layout = Layout::new(Document::new(), PageSize::A4, Margins::all(50.0)).unwrap();

    let f = font.clone();
    layout.on_footer(move |page, info, m| {
        let s = format!("Page {} of {}", info.number, info.total);
        let w = f.measure(&s, 9.0);
        page.draw_text(
            info.size.width - m.right - w,
            info.size.height - 25.0,
            &f,
            9.0,
            Color::BLACK,
            &s,
        );
    });

    layout
        .add(Block::paragraph(
            "Quarterly report. ".repeat(300),
            body.clone(),
        ))
        .unwrap();

    let mut table = Table::new(
        vec![
            Column::Weight(3.0),
            Column::Weight(1.0),
            Column::Fixed(80.0),
        ],
        body.clone(),
    );
    table.header = Some(vec![
        Cell::new("Item").background(Color::rgb(230, 230, 230)),
        Cell::new("Qty")
            .align(Align::Right)
            .background(Color::rgb(230, 230, 230)),
        Cell::new("Total")
            .align(Align::Right)
            .background(Color::rgb(230, 230, 230)),
    ]);
    for i in 0..120 {
        table.rows.push(vec![
            Cell::new(format!(
                "Line item {i} with a fairly long description that may wrap"
            )),
            Cell::new(format!("{i}")).align(Align::Right),
            Cell::new(format!("{:.2}", i as f32 * 3.5)).align(Align::Right),
        ]);
    }
    layout.add(Block::Table(table)).unwrap();

    let doc = layout.finish().unwrap();
    assert!(doc.page_count() > 3, "got {} pages", doc.page_count());
    let bytes = doc.finish().unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
    if let Ok(dir) = std::env::var("MKPDF_OUT") {
        std::fs::write(format!("{dir}/layout.pdf"), &bytes).unwrap();
    }
}

#[test]
fn table_column_mismatch_is_an_error() {
    let font = font();
    let style = TextStyle::new(font, 10.0);
    let mut layout = Layout::new(Document::new(), PageSize::A4, Margins::all(40.0)).unwrap();
    let mut t = Table::new(vec![Column::Weight(1.0), Column::Weight(1.0)], style);
    t.rows.push(vec![Cell::new("only one")]);
    assert!(matches!(
        layout.add(Block::Table(t)),
        Err(Error::ColumnMismatch { .. })
    ));
}

#[test]
fn empty_layout_yields_one_page() {
    let layout = Layout::new(Document::new(), PageSize::LETTER, Margins::all(36.0)).unwrap();
    assert_eq!(layout.finish().unwrap().page_count(), 1);
}

#[test]
fn rejects_margins_larger_than_page() {
    assert!(Layout::new(Document::new(), PageSize::A4, Margins::all(400.0)).is_err());
}
