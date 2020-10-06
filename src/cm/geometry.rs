#[derive(Clone, Copy)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl Rect {
    pub fn remove_rows_from_top(self, mut n: usize) -> Rect {
        if n > self.h {
            n = self.h;
        }

        Rect {
            x: self.x,
            y: self.y + n,
            w: self.w,
            h: self.h - n,
        }
    }

    pub fn horizontal_split(self, denominator: usize) -> (Rect, Rect) {
        assert!(denominator > 0);
        let h = self.h / denominator * (denominator - 1);
        (
            Rect {
                x: self.x,
                y: self.y,
                w: self.w,
                h,
            },
            Rect {
                x: self.x,
                y: self.y + h,
                w: self.w,
                h: self.h - h,
            },
        )
    }

    pub fn vertical_split(self, denominator: usize) -> (Rect, Rect, Rect) {
        assert!(denominator > 0);
        let w = self.w / denominator;
        (
            Rect {
                x: self.x,
                y: self.y,
                w,
                h: self.h,
            },
            Rect {
                x: self.x + w,
                y: self.y,
                w: self.w - w,
                h: self.h,
            },
            Rect {
                x: self.x + 2 * w,
                y: self.y,
                w: self.w - 2 * w,
                h: self.h,
            },
        )
    }
}

#[derive(Clone, Copy)]
pub struct Row {
    pub x: usize,
    pub y: usize,
    pub w: usize,
}
