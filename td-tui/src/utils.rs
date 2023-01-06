use tui::layout::Rect;

pub trait RectExt {
    fn take_x(self, amount: u16) -> Self;
    fn skip_x(self, amount: u16) -> Self;
    fn take_y(self, amount: u16) -> Self;
    fn skip_y(self, amount: u16) -> Self;

    /// Creates a rect in the center of this one.
    fn center_rect(&self, width: u16, height: u16) -> Self;
}

impl RectExt for Rect {
    fn take_x(self, amount: u16) -> Self {
        Self::new(self.x, self.y, amount, self.height)
    }

    fn skip_x(self, amount: u16) -> Self {
        Self::new(self.x + amount, self.y, self.width - amount, self.height)
    }

    fn take_y(self, amount: u16) -> Self {
        Self::new(self.x, self.y, self.width, amount)
    }

    fn skip_y(self, amount: u16) -> Self {
        Self::new(self.x, self.y + amount, self.width, self.height - amount)
    }

    fn center_rect(&self, width: u16, height: u16) -> Self {
        let center_x = self.x + self.width / 2;
        let center_y = self.y + self.height / 2;
        Self {
            x: center_x - width / 2,
            y: center_y - height / 2,
            width,
            height,
        }
    }
}
