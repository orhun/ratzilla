use ratatui::{buffer::Buffer, layout::Rect, style::Modifier, text::Span, widgets::Widget};

/// When added as a modifier to a style, the styled element is marked as hyperlink.
pub(crate) const HYPERLINK: Modifier = Modifier::SLOW_BLINK;

pub struct Hyperlink<'a> {
    line: Span<'a>,
}

impl<'a> Hyperlink<'a> {
    pub fn new(url: &'a str) -> Self {
        Self {
            line: Span::from(url).style(HYPERLINK),
        }
    }
}

impl<'a> Widget for Hyperlink<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.line.render(area, buf);
    }
}
