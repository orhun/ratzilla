use ratatui::style::Modifier;

/// Supported cursor shapes.
#[derive(Debug, Default)]
pub enum CursorShape {
    /// A non blinking block cursor shape (█).
    #[default]
    SteadyBlock,
    /// A non blinking underscore cursor shape (_).
    SteadyUnderScore,
}

impl CursorShape {
    /// Transforms the given style to hide the cursor.
    pub fn hide(&self, style: Modifier) -> Modifier {
        match self {
            CursorShape::SteadyBlock => style ^ Modifier::REVERSED,
            CursorShape::SteadyUnderScore => style ^ Modifier::UNDERLINED,
        }
    }

    /// Transforms the given style to show the cursor.
    pub fn show(&self, style: Modifier) -> Modifier {
        match self {
            CursorShape::SteadyBlock => style | Modifier::REVERSED,
            CursorShape::SteadyUnderScore => style | Modifier::UNDERLINED,
        }
    }
}
