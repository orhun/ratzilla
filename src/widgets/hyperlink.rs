use log::info;
use ratatui::{buffer::Buffer, layout::Rect, style::Style, text::Span, widgets::Widget};

/// A widget that can be used to render hyperlinks.
///
/// ```rust no_run
/// use ratzilla::widgets::Hyperlink;
///
/// let link = Hyperlink::new("https://ratatui.rs");
///
/// // Then you can render it as usual:
/// // frame.render_widget(link, frame.area());
/// ```
#[derive(Debug)]
pub struct Hyperlink<'a> {
    /// Line.
    line: Span<'a>,
}

impl<'a> Hyperlink<'a> {
    /// Constructs a new [`Hyperlink`] widget.
    pub fn new<T, U>(content: T, url: U) -> Self
    where
        T: Into<Span<'a>>,
        U: Into<&'static str>,
    {
        let line = content
            .into()
            .patch_style(Style::new().hyperlink(url.into()));
        // info!("span: {:#?}", line);
        Self { line }
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
