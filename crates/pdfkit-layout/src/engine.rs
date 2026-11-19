use pdfkit_core::{Document, Page, PageSize};

use crate::text::wrap;
use crate::types::*;

type Decorator = Box<dyn Fn(&mut Page, PageInfo, &Margins)>;

pub struct Layout {
    doc: Document,
    size: PageSize,
    margins: Margins,
    page: Option<usize>,
    y: f32,
    header: Option<Decorator>,
    footer: Option<Decorator>,
}

impl Layout {
    pub fn new(doc: Document, size: PageSize, margins: Margins) -> Result<Self> {
        let l = Self { doc, size, margins, page: None, y: margins.top, header: None, footer: None };
        if l.content_width() <= 0.0 || l.bottom() <= l.margins.top {
            return Err(Error::NoContentArea);
        }
        Ok(l)
    }

    /// Called for every page after layout, so `PageInfo::total` is known.
    /// Draw inside the margins; the callback receives them.
    pub fn on_header(&mut self, f: impl Fn(&mut Page, PageInfo, &Margins) + 'static) {
        self.header = Some(Box::new(f));
    }

    pub fn on_footer(&mut self, f: impl Fn(&mut Page, PageInfo, &Margins) + 'static) {
        self.footer = Some(Box::new(f));
    }

    pub fn content_width(&self) -> f32 {
        self.size.width - self.margins.left - self.margins.right
    }

    fn bottom(&self) -> f32 {
        self.size.height - self.margins.bottom
    }

    fn at_top(&self) -> bool {
        self.y <= self.margins.top + f32::EPSILON
    }

    fn new_page(&mut self) -> Result<()> {
        self.doc.add_page(self.size)?;
        self.page = Some(self.doc.page_count() - 1);
        self.y = self.margins.top;
        Ok(())
    }

    fn page(&mut self) -> Result<&mut Page> {
        if self.page.is_none() {
            self.new_page()?;
        }
        Ok(self.doc.page_mut(self.page.unwrap()).expect("page index valid"))
    }

    /// Ensure `h` points fit on the current page, starting a new page if not.
    /// Returns true if a page break happened.
    fn ensure(&mut self, h: f32) -> Result<bool> {
        if self.page.is_none() {
            self.new_page()?;
            return Ok(false);
        }
        if self.y + h > self.bottom() && !self.at_top() {
            self.new_page()?;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn add(&mut self, block: Block) -> Result<()> {
        match block {
            Block::Paragraph { text, style, align, space_after } => {
                self.paragraph(&text, &style, align, space_after)
            }
            Block::Table(t) => self.table(&t),
            Block::Image { image, width, align, space_after } => {
                self.image(&image, width, align, space_after)
            }
            Block::Spacer(h) => {
                self.page()?;
                self.y += h;
                if self.y > self.bottom() {
                    self.new_page()?;
                }
                Ok(())
            }
            Block::PageBreak => self.new_page(),
        }
    }

    fn paragraph(&mut self, text: &str, style: &TextStyle, align: Align, space_after: f32) -> Result<()> {
        let lh = style.size * style.line_height;
        let lines = wrap(text, &style.font, style.size, self.content_width());
        let x0 = self.margins.left;
        let cw = self.content_width();
        let asc = style.font.ascent(style.size);
        let desc = style.font.descent(style.size);

        for line in lines {
            self.ensure(lh)?;
            let y = self.y;
            let w = style.font.measure(&line, style.size);
            let x = aligned_x(x0, cw, w, align);
            let baseline = y + (lh - (asc + desc)) / 2.0 + asc;
            if !line.is_empty() {
                self.page()?.draw_text(x, baseline, &style.font, style.size, style.color, &line);
            }
            self.y += lh;
        }
        self.y += space_after;
        Ok(())
    }

    fn image(&mut self, image: &pdfkit_core::Image, width: Option<f32>, align: Align, space_after: f32) -> Result<()> {
        let (pw, ph) = image.size();
        if pw == 0 || ph == 0 {
            return Ok(());
        }
        let ratio = ph as f32 / pw as f32;
        let mut w = width.unwrap_or(pw as f32).min(self.content_width());
        let mut h = w * ratio;
        let avail = self.bottom() - self.margins.top;
        if h > avail {
            h = avail;
            w = h / ratio;
        }
        self.ensure(h)?;
        let x = aligned_x(self.margins.left, self.content_width(), w, align);
        let y = self.y;
        self.page()?.draw_image(x, y, w, h, image);
        self.y += h + space_after;
        Ok(())
    }

    fn table(&mut self, t: &Table) -> Result<()> {
        if t.columns.is_empty() {
            return Err(Error::EmptyTable);
        }
        for (i, r) in t.header.iter().chain(t.rows.iter()).enumerate() {
            if r.len() != t.columns.len() {
                return Err(Error::ColumnMismatch { row: i, got: r.len(), expected: t.columns.len() });
            }
        }
        let widths = column_widths(&t.columns, self.content_width());

        if let Some(h) = &t.header {
            let rh = row_height(h, &widths, t);
            self.ensure(rh)?;
            self.draw_row(h, &widths, t, rh)?;
        }
        for row in &t.rows {
            let rh = row_height(row, &widths, t);
            // Keep the header row together with at least the first data row.
            if self.ensure(rh)? {
                if let Some(h) = &t.header {
                    let hh = row_height(h, &widths, t);
                    self.draw_row(h, &widths, t, hh)?;
                }
            }
            self.draw_row(row, &widths, t, rh)?;
        }
        Ok(())
    }

    fn draw_row(&mut self, row: &Row, widths: &[f32], t: &Table, rh: f32) -> Result<()> {
        let y = self.y;
        let mut x = self.margins.left;
        for (cell, &w) in row.iter().zip(widths) {
            let style = cell.style.as_ref().unwrap_or(&t.style);
            let lh = style.size * style.line_height;
            let asc = style.font.ascent(style.size);
            let desc = style.font.descent(style.size);
            let inner = (w - 2.0 * t.padding).max(1.0);
            let lines = wrap(&cell.text, &style.font, style.size, inner);

            let page = self.page()?;
            if let Some(bg) = cell.background {
                page.fill_rect(x, y, w, rh, bg);
            }
            for (i, line) in lines.iter().enumerate() {
                if line.is_empty() {
                    continue;
                }
                let lw = style.font.measure(line, style.size);
                let tx = aligned_x(x + t.padding, inner, lw, cell.align);
                let baseline = y + t.padding + i as f32 * lh + (lh - (asc + desc)) / 2.0 + asc;
                page.draw_text(tx, baseline, &style.font, style.size, style.color, line);
            }
            if let Some((color, bw)) = t.border {
                page.stroke_rect(x, y, w, rh, color, bw);
            }
            x += w;
        }
        self.y += rh;
        Ok(())
    }

    pub fn finish(mut self) -> Result<Document> {
        if self.page.is_none() {
            self.new_page()?;
        }
        let total = self.doc.page_count();
        for i in 0..total {
            let info = PageInfo { number: i + 1, total, size: self.size };
            let page = self.doc.page_mut(i).expect("page index valid");
            if let Some(f) = &self.header {
                f(page, info, &self.margins);
            }
            if let Some(f) = &self.footer {
                f(page, info, &self.margins);
            }
        }
        Ok(self.doc)
    }
}

fn aligned_x(x0: f32, width: f32, content: f32, align: Align) -> f32 {
    match align {
        Align::Left => x0,
        Align::Center => x0 + (width - content) / 2.0,
        Align::Right => x0 + width - content,
    }
}

fn column_widths(cols: &[Column], total: f32) -> Vec<f32> {
    let fixed: f32 = cols.iter().filter_map(|c| if let Column::Fixed(w) = c { Some(*w) } else { None }).sum();
    let weights: f32 = cols.iter().filter_map(|c| if let Column::Weight(w) = c { Some(*w) } else { None }).sum();
    let rest = (total - fixed).max(0.0);
    cols.iter()
        .map(|c| match c {
            Column::Fixed(w) => *w,
            Column::Weight(w) if weights > 0.0 => rest * w / weights,
            Column::Weight(_) => 0.0,
        })
        .collect()
}

fn row_height(row: &Row, widths: &[f32], t: &Table) -> f32 {
    row.iter()
        .zip(widths)
        .map(|(cell, &w)| {
            let style = cell.style.as_ref().unwrap_or(&t.style);
            let inner = (w - 2.0 * t.padding).max(1.0);
            let n = wrap(&cell.text, &style.font, style.size, inner).len().max(1);
            n as f32 * style.size * style.line_height + 2.0 * t.padding
        })
        .fold(0.0, f32::max)
}
