use mkpdf_core::Font;

/// Greedy word wrap. Honors `\n`; words wider than `max` are split by character.
pub fn wrap(text: &str, font: &Font, size: f32, max: f32) -> Vec<String> {
    let space = font.measure(" ", size);
    let mut lines = Vec::new();

    for para in text.split('\n') {
        let mut cur = String::new();
        let mut cur_w = 0.0f32;

        for word in para.split(' ').filter(|w| !w.is_empty()) {
            let w = font.measure(word, size);
            if w > max {
                if !cur.is_empty() {
                    lines.push(std::mem::take(&mut cur));
                }
                let mut pieces = split_long(word, font, size, max);
                let last = pieces.pop().unwrap_or_default();
                lines.extend(pieces);
                cur_w = font.measure(&last, size);
                cur = last;
                continue;
            }
            if cur.is_empty() {
                cur.push_str(word);
                cur_w = w;
            } else if cur_w + space + w <= max {
                cur.push(' ');
                cur.push_str(word);
                cur_w += space + w;
            } else {
                lines.push(std::mem::take(&mut cur));
                cur.push_str(word);
                cur_w = w;
            }
        }
        lines.push(cur);
    }
    lines
}

fn split_long(word: &str, font: &Font, size: f32, max: f32) -> Vec<String> {
    let mut out = Vec::new();
    let mut piece = String::new();
    for ch in word.chars() {
        piece.push(ch);
        if piece.chars().count() > 1 && font.measure(&piece, size) > max {
            piece.pop();
            out.push(std::mem::take(&mut piece));
            piece.push(ch);
        }
    }
    if !piece.is_empty() {
        out.push(piece);
    }
    out
}
