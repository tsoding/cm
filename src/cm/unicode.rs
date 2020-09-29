use std::ops::Range;
use unicode_width::UnicodeWidthChar;

pub fn width_substr(s: &str, range: Range<usize>) -> Option<(&str, (usize, usize))> {
    let w = range.end - range.start;
    let mut start = s.chars();
    let mut start_bytes: usize = 0;

    // .skip(self.cursor_x)
    let left_padding: usize = {
        let mut m: i32 = range.start as i32;
        while m > 0 {
            match start.next().map(|x| (x.width().unwrap_or(0), x.len_utf8())) {
                Some((boxes, bytes)) => {
                    m -= boxes as i32;
                    start_bytes += bytes;
                },
                _ => break,
            }
        }
        if m <= 0 {
            i32::abs(m) as usize
        } else {
            0
        }
    };

    let mut end_bytes: usize = start_bytes;

    // .take(w)
    let right_padding: usize = {
        let mut m = w;
        while m > 0 {
            match start.next().map(|x| (x.width().unwrap_or(0), x.len_utf8())) {
                Some((boxes, bytes)) if boxes <= m => {
                    m -= boxes;
                    end_bytes += bytes;
                }
                _ => break,
            }
        }
        m as usize
    };

    s.get(start_bytes..end_bytes).map(|x| (x, (left_padding, right_padding)))
}
