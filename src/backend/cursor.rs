use ratatui::style::{Style, Stylize};

/// Supported cursor shapes.
#[derive(Debug, Default)]
pub enum CursorShape {
    /// A non blinking block cursor shape (■).
    #[default]
    SteadyBlock,
    /// A non blinking underscore cursor shape (_).
    SteadyUnderScore,
}

impl CursorShape {
    /// Transforms the given style to hide the cursor.
    pub fn hide(&self, style: Style) -> Style {
        match self {
            CursorShape::SteadyBlock => style.not_reversed(),
            CursorShape::SteadyUnderScore => style.not_underlined(),
        }
    }

    /// Transforms the given style to show the cursor.
    pub fn show(&self, style: Style) -> Style {
        match self {
            CursorShape::SteadyBlock => style.reversed(),
            CursorShape::SteadyUnderScore => style.underlined(),
        }
    }
}
