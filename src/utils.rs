use ratatui::{buffer::Cell, style::Color};
use web_sys::{
    wasm_bindgen::{JsCast, JsValue},
    window, Document, Element, HtmlStyleElement,
};

pub(crate) fn create_span(cell: &Cell) -> Element {
    let document = web_sys::window().unwrap().document().unwrap();
    let span = document.create_element("span").unwrap();
    span.set_inner_html(cell.symbol());

    // let style = get_cell_color(cell);

    let class = format!(
        "{} {}",
        ansi_to_class(cell.fg, false),
        ansi_to_class(cell.bg, true)
    );
    span.set_attribute("class", &class).unwrap();
    span
}

pub(crate) fn get_cell_color(cell: &Cell) -> String {
    let fg = ansi_to_rgb(cell.fg);
    let bg = ansi_to_rgb(cell.bg);

    let fg_style = match fg {
        Some(color) => format!("color: rgb({}, {}, {});", color.0, color.1, color.2),
        None => "color: rgb(255, 255, 255);".to_string(),
    };

    let bg_style = match bg {
        Some(color) => format!(
            "background-color: rgb({}, {}, {});",
            color.0, color.1, color.2
        ),
        None => "background-color: transparent;".to_string(),
    };

    format!("{} {}", fg_style, bg_style)
}

pub fn ansi_to_rgb(color: Color) -> Option<(u8, u8, u8)> {
    match color {
        Color::Black => Some((0, 0, 0)),
        Color::Red => Some((128, 0, 0)),
        Color::Green => Some((0, 128, 0)),
        Color::Yellow => Some((128, 128, 0)),
        Color::Blue => Some((0, 0, 128)),
        Color::Magenta => Some((128, 0, 128)),
        Color::Cyan => Some((0, 128, 128)),
        Color::Gray => Some((192, 192, 192)),
        Color::DarkGray => Some((128, 128, 128)),
        Color::LightRed => Some((255, 0, 0)),
        Color::LightGreen => Some((0, 255, 0)),
        Color::LightYellow => Some((255, 255, 0)),
        Color::LightBlue => Some((0, 0, 255)),
        Color::LightMagenta => Some((255, 0, 255)),
        Color::LightCyan => Some((0, 255, 255)),
        Color::White => Some((255, 255, 255)),
        _ => None, // Handle invalid color names
    }
}

pub fn set_document_title(title: &str) {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .set_title(title);
}

/// append css string to the document head
pub(crate) fn inject_css(css: &str) -> Result<(), JsValue> {
    // Get the global window and document
    let document = window().unwrap().document().unwrap();
    let style_element = document
        .create_element("style")?
        .dyn_into::<HtmlStyleElement>()?;
    style_element.set_type("text/css");
    style_element.set_inner_html(css);
    document.head().unwrap().append_child(&style_element)?;

    Ok(())
}

/// convert a color to a css class name and return it as a string
/// if background is true, the class name will be in the format .bg-{color_name}
/// otherwise, it will be in the format .fg-{color_name}
pub fn ansi_to_class(color: Color, background: bool) -> String {
    let color_name = match color {
        Color::Black => String::from("black"),
        Color::Red => String::from("red"),
        Color::Green => String::from("green"),
        Color::Yellow => String::from("yellow"),
        Color::Blue => String::from("blue"),
        Color::Magenta => String::from("magenta"),
        Color::Cyan => String::from("cyan"),
        Color::Gray => String::from("gray"),
        Color::DarkGray => String::from("dark_gray"),
        Color::LightRed => String::from("light_red"),
        Color::LightGreen => String::from("light_green"),
        Color::LightYellow => String::from("light_yellow"),
        Color::LightBlue => String::from("light_blue"),
        Color::LightMagenta => String::from("light_magenta"),
        Color::LightCyan => String::from("light_cyan"),
        Color::White => String::from("white"),
        _ => String::from("default"), // Handle unset colors as default color
    };
    if background {
        format!("bg-{color_name}")
    } else {
        format!("fg-{color_name}")
    }
}

pub struct CssMode {
    pub background: (u8, u8, u8),
    pub foreground: (u8, u8, u8),
}
// struct for adjusting base styles for a specific mode (e.g. dark mode/light mode)
// this basically defines just default colors for the mode: default foreground and background colors
pub struct CssModeColors {
    pub light: CssMode,
    pub dark: CssMode,
}

impl Default for CssModeColors {
    fn default() -> Self {
        Self {
            // as rule of thumb, the default foreground color should be black and the default background color should be white
            // but it's also not wise to use **absolute** white/black as the default colors, so we use a slightly lighter/darker version of the color
            // https://graphicdesign.stackexchange.com/questions/25356/why-not-use-pure-black-000-and-pure-white-fff
            light: CssMode {
                background: (231, 231, 231),
                foreground: (61, 61, 61),
            },
            dark: CssMode {
                background: (61, 61, 61),
                foreground: (231, 231, 231),
            },
        }
    }
}
/// struct for adjusting base styles
/// this can be used to adjust the base styles of the app (e.g. set you own theme)
/// you can basically overload the default colors and add your own
/// you can also add your own css rules
///
///
pub struct CssStyle {
    pub colors: Vec<(Color, (u8, u8, u8))>,
    pub modes: CssModeColors,
}

impl Default for CssStyle {
    fn default() -> Self {
        Self {
            modes: CssModeColors::default(),
            colors: vec![
                (Color::Black, (0, 0, 0)),
                (Color::Red, (128, 0, 0)),
                (Color::Green, (0, 128, 0)),
                (Color::Yellow, (128, 128, 0)),
                (Color::Blue, (0, 0, 128)),
                (Color::Magenta, (128, 0, 128)),
                (Color::Cyan, (0, 128, 128)),
                (Color::Gray, (192, 192, 192)),
                (Color::DarkGray, (128, 128, 128)),
                (Color::LightRed, (255, 0, 0)),
                (Color::LightGreen, (0, 255, 0)),
                (Color::LightYellow, (255, 255, 0)),
                (Color::LightBlue, (0, 0, 255)),
                (Color::LightMagenta, (255, 0, 255)),
                (Color::LightCyan, (0, 255, 255)),
                (Color::White, (255, 255, 255)),
            ],
        }
    }
}

pub(crate) fn inject_base_style(style_options: CssStyle) -> Result<(), JsValue> {
    // loop through all the colors and create a css rule for each one

    // String to hold the css rules for all class names
    let mut style: String = String::from("");

    for color in style_options.colors {
        let color_value = color.1;
        let color_name = color.0.to_string().to_lowercase();

        // create a css rule for the color class name and add it to the style string
        // both for foreground and background
        let css_rule_fg = format!(
            ".fg-{color_name} {{ color: rgb({}, {}, {}); }}\n",
            color_value.0, color_value.1, color_value.2
        );
        let css_rule_bg = format!(
            ".bg-{color_name} {{ background-color: rgb({}, {}, {}); }}\n",
            color_value.0, color_value.1, color_value.2
        );

        style.push_str(&css_rule_fg);
        style.push_str(&css_rule_bg);
    }

    // add a default color class name for unset colors
    style.push_str(".bg-default { background-color: transparent; }");

    // define light mode and dark mode styles
    let light_fg = style_options.modes.light.foreground;
    let light_bg = style_options.modes.light.background;
    let default_colors_light = format!(
        r#"
        @media (prefers-color-scheme: light) {{
            .fg-default {{ color: rgb({}, {}, {}); }}
         
    }}
    "#,
        light_fg.0, light_fg.1, light_fg.2
    );

    let default_bg_light = format!(
        r#"
        @media (prefers-color-scheme: light) {{
            body {{ background-color: rgb({}, {}, {}); }}
         
    }}"#,
        light_bg.0, light_bg.1, light_bg.2
    );

    style.push_str(&default_colors_light);
    style.push_str(&default_bg_light);

    let dark_fg = style_options.modes.dark.foreground;
    let dark_bg = style_options.modes.dark.background;
    let default_colors_dark = format!(
        r#"
        @media (prefers-color-scheme: dark) {{
            .fg-default {{ color: rgb({}, {}, {}); }}
         
    }}
    "#,
        dark_fg.0, dark_fg.1, dark_fg.2
    );
    let default_bg_dark = format!(
        r#"
        @media (prefers-color-scheme: dark) {{
            body {{ background-color: rgb({}, {}, {}); }}
         
    }}"#,
        dark_bg.0, dark_bg.1, dark_bg.2
    );
    style.push_str(&default_bg_dark);
    style.push_str(&default_colors_dark);

    // injet rules for all <pre> elements in #grid div element in the document
    style.push('\n');
    style.push_str("#grid pre { margin: 0px; }");
    // inject the css rules into the document head
    inject_css(&style)?;
    Ok(())
}
