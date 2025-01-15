use ratatui::{buffer::Cell, style::Color};
use web_sys::{wasm_bindgen::JsValue, Document, Element, HtmlCanvasElement};

pub(crate) fn create_span(document: &Document, cell: &Cell) -> Element {
    let span = document.create_element("span").unwrap();
    span.set_inner_html(cell.symbol());

    let style = get_cell_color_as_css(cell);
    span.set_attribute("style", &style).unwrap();
    span
}

pub(crate) fn create_anchor(document: &Document, cells: &[Cell]) -> Element {
    let anchor = document.create_element("a").unwrap();
    anchor
        .set_attribute(
            "href",
            &cells.iter().map(|c| c.symbol()).collect::<String>(),
        )
        .unwrap();
    anchor
        .set_attribute("style", &get_cell_color_as_css(&cells[0]))
        .unwrap();
    anchor
}

pub(crate) fn get_cell_color_as_css(cell: &Cell) -> String {
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
        })
        .unwrap_or((120, 120))
}

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

pub(crate) fn get_sized_buffer() -> Vec<Vec<Cell>> {
    let (width, height) = if is_mobile() {
        get_screen_size()
    } else {
        get_window_size()
    };
    vec![vec![Cell::default(); width as usize]; height as usize]
}

pub(crate) fn get_sized_buffer_from_canvas(canvas: &HtmlCanvasElement) -> Vec<Vec<Cell>> {
    let width = canvas.client_width() as u16 / 10_u16;
    let height = canvas.client_height() as u16 / 19_u16;
    vec![vec![Cell::default(); width as usize]; height as usize]
}

pub fn set_document_title(title: &str) {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .set_title(title);
}
