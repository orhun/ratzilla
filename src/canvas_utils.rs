use ratatui::{buffer::Cell, style::Color};
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};
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

pub(crate) fn get_sized_buffer_canvas(canvas: &HtmlCanvasElement) -> Vec<Vec<Cell>> {
    let widthh = canvas.client_width() as u16 / 10 as u16;
    let heighth = canvas.client_height() as u16 / 19 as u16;
    let width = widthh as usize;
    let height = heighth as usize;

    vec![vec![Cell::default(); width as usize]; height as usize]
}

pub(crate) fn get_cell_color_canvas(cell: &Cell) -> (String, String) {
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
