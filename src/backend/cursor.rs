use ratatui::style::{Style, Stylize};

/// Supported cursor shapes.
#[derive(Debug, Default)]
pub enum CursorShape {
    /// A non blinking block cursor shape (█).
    #[default]
    SteadyBlock,
    /// A non blinking underscore cursor shape (_).
    SteadyUnderScore,
    /// This variant is only used to clear cursor.
    None,
}

impl CursorShape {
    /// Transforms the given style to hide the cursor.
    pub fn hide(&self, style: Style) -> Style {
        match self {
            CursorShape::SteadyBlock => style.not_reversed(),
            CursorShape::SteadyUnderScore => style.not_underlined(),
            CursorShape::None => style,
        }
    }

    /// Transforms the given style to show the cursor.
    pub fn show(&self, style: Style) -> Style {
        match self {
            CursorShape::SteadyBlock => style.reversed(),
            CursorShape::SteadyUnderScore => style.underlined(),
            CursorShape::None => style,
        }
    }

    /// Returns a list of css fields and their values for this cursor shape.
    pub fn get_css_field_value(&self) -> Vec<(String, Option<String>)> {
        match self {
            CursorShape::SteadyBlock => vec![
                ("cursor".to_string(), Some("block".to_string())),
                ("text-decoration".to_string(), Some("none".to_string())),
            ],
            CursorShape::SteadyUnderScore => vec![
                ("cursor".to_string(), Some("underscore".to_string())),
                ("text-decoration".to_string(), Some("underline".to_string())),
            ],
            CursorShape::None => vec![
                ("cursor".to_string(), None),
                ("text-decoration".to_string(), None),
            ],
        }
    }
}
