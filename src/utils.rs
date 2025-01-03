use ratatui::{buffer::Cell, style::Color};
use web_sys::Element;

pub fn create_cell(cell: &Cell) -> Element {
    let document = web_sys::window().unwrap().document().unwrap();
    let span = document.create_element("span").unwrap();
    span.set_inner_html(cell.symbol());
    let fg = ansi_to_rgb(cell.fg);
    let bg = ansi_to_rgb(cell.bg);

    let fg_style = match fg {
        Some(color) => format!("color: rgb({}, {}, {});", color.0, color.1, color.2),
        None => {
            web_sys::console::log_1(&"Invalid color".into());
            "color: rgb(255, 255, 255);".to_string()
        }
    };

    let bg_style = match bg {
        Some(color) => format!(
            "background-color: rgb({}, {}, {});",
            color.0, color.1, color.2
        ),
        None => {
            web_sys::console::log_1(&"Invalid color".into());
            "background-color: transparent;".to_string()
        }
    };

    let style = format!("{} {}", fg_style, bg_style);

    span.set_attribute("style", &style).unwrap();
    span
    // let pre = document.create_element("pre").unwrap();
    // pre.set_attribute("style", "margin: 0px;").unwrap();
    // pre.append_child(&span).unwrap();

    // pre
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
