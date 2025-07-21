use crate::{
    backend::color::ansi_to_rgb,
    error::Error,
    event::MouseEvent,
    utils::{get_screen_size, get_window_size, is_mobile},
};
use compact_str::{format_compact, CompactString};
use ratatui::{
    buffer::Cell,
    style::{Color, Modifier},
};
use std::fmt::Debug;
use web_sys::{
    wasm_bindgen::{JsCast, JsValue},
    window, Document, Element, HtmlCanvasElement, Window,
};

/// A handler for mouse events.
///
/// This wrapper allows structs containing mouse event callbacks to derive Debug
/// by providing a Debug implementation that doesn't expose the closure internals.
pub(super) struct MouseEventHandler {
    callback: Box<dyn FnMut(MouseEvent) + 'static>,
}

impl MouseEventHandler {
    /// Creates a new `MouseEventHandler` with the given callback.
    pub fn new<F>(handler: F) -> Self
    where
        F: FnMut(MouseEvent) + 'static,
    {
        Self {
            callback: Box::new(handler),
        }
    }

    /// Invokes the callback with the given mouse event.
    pub fn call(&mut self, event: MouseEvent) {
        (self.callback)(event);
    }
}

impl Debug for MouseEventHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MouseEventHandler").finish_non_exhaustive()
    }
}

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
    let mut fg = ansi_to_rgb(cell.fg);
    let mut bg = ansi_to_rgb(cell.bg);

    if cell.modifier.contains(Modifier::REVERSED) {
        std::mem::swap(&mut fg, &mut bg);
    }

    let fg_style = match fg {
        Some(color) => format!("color: rgb({}, {}, {});", color.0, color.1, color.2),
        None => "color: rgb(255, 255, 255);".to_string(),
    };

    let bg_style = match bg {
        Some(color) => format!(
            "background-color: rgb({}, {}, {});",
            color.0, color.1, color.2
        ),
        None => {
            // If the cell needs to be reversed but we don't have a valid background,
            // then default the background to white.
            if cell.modifier.contains(Modifier::REVERSED) {
                "background-color: rgb(255, 255, 255);".to_string()
            } else {
                "background-color: transparent;".to_string()
            }
        }
    };

    let mut modifier_style = String::new();
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

/// Converts a Color to a CSS style.
pub(crate) fn get_canvas_color(color: Color, fallback_color: Color) -> CompactString {
    let color = ansi_to_rgb(color).unwrap_or_else(|| ansi_to_rgb(fallback_color).unwrap());

    format_compact!("rgb({}, {}, {})", color.0, color.1, color.2)
}

/// Calculates the number of pixels that can fit in the window.
pub(crate) fn get_raw_window_size() -> (u16, u16) {
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

/// Returns the number of pixels that can fit in the window.
pub(crate) fn get_raw_screen_size() -> (i32, i32) {
    let s = web_sys::window().unwrap().screen().unwrap();
    (s.width().unwrap(), s.height().unwrap())
}

/// Returns a buffer based on the screen size.
pub(crate) fn get_sized_buffer() -> Vec<Vec<Cell>> {
    let size = if is_mobile() {
        get_screen_size()
    } else {
        get_window_size()
    };
    vec![vec![Cell::default(); size.width as usize]; size.height as usize]
}

/// Returns a buffer based on the canvas size.
pub(crate) fn get_sized_buffer_from_canvas(canvas: &HtmlCanvasElement) -> Vec<Vec<Cell>> {
    let width = canvas.client_width() as u16 / 10_u16;
    let height = canvas.client_height() as u16 / 19_u16;
    vec![vec![Cell::default(); width as usize]; height as usize]
}

/// Returns the document object from the window.
pub(crate) fn get_document() -> Result<Document, Error> {
    get_window()?
        .document()
        .ok_or(Error::UnableToRetrieveDocument)
}

/// Returns the window object.
pub(crate) fn get_window() -> Result<Window, Error> {
    window().ok_or(Error::UnableToRetrieveWindow)
}

/// Returns an element by its ID or the body element if no ID is provided.
pub(crate) fn get_element_by_id_or_body(id: Option<&String>) -> Result<web_sys::Element, Error> {
    match id {
        Some(id) => get_document()?
            .get_element_by_id(id)
            .ok_or_else(|| Error::UnableToRetrieveElementById(id.to_string())),
        None => get_document()?
            .body()
            .ok_or(Error::UnableToRetrieveBody)
            .map(|body| body.into()),
    }
}

/// Returns the performance object from the window.
pub(crate) fn performance() -> Result<web_sys::Performance, Error> {
    Ok(get_window()?
        .performance()
        .ok_or(Error::UnableToRetrieveComponent("Performance"))?)
}

/// Creates a new canvas element in the specified parent element with the
/// given width and height.
pub(crate) fn create_canvas_in_element(
    parent: &Element,
    width: u32,
    height: u32,
) -> Result<HtmlCanvasElement, Error> {
    let element = get_document()?.create_element("canvas")?;

    let canvas = element
        .clone()
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| ())
        .expect("Unable to cast canvas element");
    canvas.set_width(width);
    canvas.set_height(height);

    parent.append_child(&element)?;

    Ok(canvas)
}

/// Measures the actual pixel size of a DOM cell by creating a temporary test element.
///
/// This function creates a span with the same styling as terminal cells,
/// measures its dimensions, and then removes it from the DOM.
pub(super) fn measure_dom_cell_size(
    document: &Document,
    grid_parent: &Element,
) -> Result<(u32, u32), Error> {
    // Create a temporary test span with a single character
    let test_span = document.create_element("span")?;
    test_span.set_inner_html("W"); // Use a wide character for consistent measurement
    test_span.set_attribute("style", "display: inline-block; visibility: hidden; position: absolute; white-space: pre; font-family: monospace;")?;

    // Create a temporary container to ensure consistent measurement
    let test_container = document.create_element("pre")?;
    test_container.set_attribute(
        "style",
        "visibility: hidden; position: absolute; margin: 0; padding: 0; line-height: normal;",
    )?;
    test_container.append_child(&test_span)?;

    // Add to DOM for measurement
    grid_parent.append_child(&test_container)?;

    // Use client dimensions for measurement
    let width = test_span.client_width() as u32;
    let height = test_span.client_height() as u32;

    // Clean up
    grid_parent.remove_child(&test_container)?;

    // Ensure we have reasonable dimensions (fallback to canvas constants if measurement fails)
    let width = if width > 0 { width } else { 10 };
    let height = if height > 0 { height } else { 19 };

    Ok((width, height))
}
