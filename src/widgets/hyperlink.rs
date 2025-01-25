use ratatui::{buffer::Buffer, layout::Rect, style::Modifier, text::Span, widgets::Widget};

/// Hyperlink modifier.
///
/// When added as a modifier to a style, the styled element is marked as
/// hyperlink.
pub(crate) const HYPERLINK_MODIFIER: Modifier = Modifier::SLOW_BLINK;

/// Hyperlink widget.
pub struct Hyperlink<'a> {
    /// Line.
    line: Span<'a>,
}

impl<'a> Hyperlink<'a> {
    /// Constructs a new [`Hyperlink`] widget.
    pub fn new<T>(url: T) -> Self
    where
        T: Into<Span<'a>>,
    {
        Self {
            line: url.into().style(HYPERLINK_MODIFIER),
        }
    }
}

impl Widget for Hyperlink<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.line.render(area, buf);
    }
}
