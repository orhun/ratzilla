use ratatui::{buffer::Cell, style::{Color, Modifier}};
use web_sys::{wasm_bindgen::JsValue, Document, Element, HtmlCanvasElement};

use crate::error::Error;

/// Creates a new `<span>` element with the given cell.
pub(crate) fn create_span(document: &Document, cell: &Cell) -> Result<Element, Error> {
    let span = document.create_element("span")?;
    span.set_inner_html(cell.symbol());

    let style = get_cell_style_as_css(cell);
    span.set_attribute("style", &style)?;
    Ok(span)
}

/// Creates a new `<a>` element with the given cells.
pub(crate) fn create_anchor(document: &Document, cells: &[Cell]) -> Result<Element, Error> {
    let anchor = document.create_element("a")?;

    anchor.set_attribute(
        "href",
        &cells.iter().map(|c| c.symbol()).collect::<String>(),
    )?;

    anchor.set_attribute("style", &get_cell_style_as_css(&cells[0]))?;
    Ok(anchor)
}

/// Converts a cell to a CSS style.
pub(crate) fn get_cell_style_as_css(cell: &Cell) -> String {
    let fg = ansi_to_rgb(cell.fg);
    let bg = ansi_to_rgb(cell.bg);
    let mut modifier_style = String::new();

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

    if cell.modifier.contains(Modifier::BOLD) {
        modifier_style.push_str("font-weight: bold; ");
    }

    if cell.modifier.contains(Modifier::DIM) {
        modifier_style.push_str("opacity: 0.5; ");
    }

    if cell.modifier.contains(Modifier::ITALIC) {
        modifier_style.push_str("font-style: italic; ");
    }

    if cell.modifier.contains(Modifier::UNDERLINED) {
        modifier_style.push_str("text-decoration: underline; ");
    }

    if cell.modifier.contains(Modifier::HIDDEN) {
        modifier_style.push_str("visibility: hidden; ");
    }

    if cell.modifier.contains(Modifier::CROSSED_OUT) {
        modifier_style.push_str("text-decoration: line-through; ");
    }

    format!("{fg_style} {bg_style} {modifier_style}")
}

/// Converts a cell to a CSS style.
pub(crate) fn get_cell_color_for_canvas(cell: &Cell) -> (String, String) {
    let fg = ansi_to_rgb(cell.fg);
    let bg = ansi_to_rgb(cell.bg);

    let fg_style = match fg {
        Some(color) => format!("rgb({}, {}, {})", color.0, color.1, color.2),
        None => "rgb(255, 255, 255)".to_string(),
    };

    let bg_style = match bg {
        Some(color) => format!("rgb({}, {}, {})", color.0, color.1, color.2),
        None => "#333".to_string(),
    };

    (fg_style, bg_style)
}

/// Converts an ANSI color to an RGB tuple.
fn ansi_to_rgb(color: Color) -> Option<(u8, u8, u8)> {
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
        Color::Rgb(r, g, b) => Some((r, g, b)),
        _ => None, // Handle invalid color names
    }
}

/// Calculates the number of characters that can fit in the window.
fn get_window_size() -> (u16, u16) {
    let (w, h) = get_raw_window_size();
    // These are mildly magical numbers... make them more precise
    (w / 10, h / 20)
}

/// Calculates the number of pixels that can fit in the window.
fn get_raw_window_size() -> (u16, u16) {
    fn js_val_to_int<I: TryFrom<usize>>(val: JsValue) -> Option<I> {
        val.as_f64().and_then(|i| I::try_from(i as usize).ok())
    }

    web_sys::window()
        .and_then(|s| {
            s.inner_width()
                .ok()
                .and_then(js_val_to_int::<u16>)
                .zip(s.inner_height().ok().and_then(js_val_to_int::<u16>))
        
        }).unwrap_or((120, 120))
}

/// Returns `true` if the screen is a mobile device.
///
// TODO: Improve this...
fn is_mobile() -> bool {
    get_raw_screen_size().0 < 550
}

/// Calculates the number of pixels that can fit in the window.
fn get_raw_screen_size() -> (i32, i32) {
    let s = web_sys::window().unwrap().screen().unwrap();
    (s.width().unwrap(), s.height().unwrap())
}

/// Calculates the number of characters that can fit in the window.
fn get_screen_size() -> (u16, u16) {
    let (w, h) = get_raw_screen_size();
    // These are mildly magical numbers... make them more precise
    (w as u16 / 10, h as u16 / 19)
}

/// Returns a buffer based on the screen size.
pub(crate) fn get_sized_buffer() -> Vec<Vec<Cell>> {
    let (width, height) = if is_mobile() {
        get_screen_size()
    } else {
        get_window_size()
    };

    vec![vec![Cell::default(); width as usize]; height as usize]
}

/// Returns a buffer based on the canvas size.
pub(crate) fn get_sized_buffer_from_canvas(canvas: &HtmlCanvasElement) -> Vec<Vec<Cell>> {
    let width = canvas.client_width() as u16 / 10_u16;
    let height = canvas.client_height() as u16 / 19_u16;

    vec![vec![Cell::default(); width as usize]; height as usize]
}
