use crate::{
    backend::color::ansi_to_rgb,
    error::Error,
    utils::{get_screen_size, get_window_size, is_mobile},
};
use compact_str::{format_compact, CompactString};
use ratatui::{
    buffer::Cell,
    style::{Color, Modifier},
};
use web_sys::{wasm_bindgen::{closure::Closure, JsCast, JsValue}, window, Document, Element, HtmlCanvasElement, HtmlElement, Window};
use crate::event::{MouseEvent, MouseEventKind};

/// Mouse events that are handled by the mouse event handlers.
const MOUSE_EVENTS: &[&str] = &[
    "mousemove",
    "mousedown",
    "mouseup",
    "mouseenter",
    "mouseleave",
    "click",
    "dblclick",
    "wheel",
];

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

/// Converts mouse coordinates to grid coordinates using element dimensions
/// This is the core function both backends use for accurate coordinate calculation
pub(super) fn mouse_to_grid_coords(
    event: &web_sys::MouseEvent,
    element: &HtmlElement,
    grid_width: u16,
    grid_height: u16,
    offset: Option<f64>,
) -> MouseEvent {
    let rect = element.get_bounding_client_rect();

    // Calculate relative position within the element
    let relative_x = (event.client_x() as f64 - rect.left() - offset.unwrap_or(0.0)).max(0.0);
    let relative_y = (event.client_y() as f64 - rect.top() - offset.unwrap_or(0.0)).max(0.0);

    // Map coordinates as fractions of element dimensions to grid coordinates
    let col = ((relative_x / rect.width()) * grid_width as f64) as u16;
    let row = ((relative_y / rect.height()) * grid_height as f64) as u16;

    // Clamp to grid bounds
    let col = col.min(grid_width.saturating_sub(1));
    let row = row.min(grid_height.saturating_sub(1));

    MouseEvent {
        kind: MouseEventKind::from(event),
        col,
        row,
        ctrl: event.ctrl_key(),
        alt: event.alt_key(),
        shift: event.shift_key(),
    }
}

/// Registers a mouse event handler for the specified element.
pub(super) fn register_mouse_event_handler(
    element: &Element,
    closure: Closure<dyn FnMut(web_sys::MouseEvent)>,
) -> Result<Closure<dyn FnMut(web_sys::MouseEvent)>, Error> {
    let closure_ref = closure.as_ref();

    MOUSE_EVENTS.iter().try_for_each(|event| {
        element
            .add_event_listener_with_callback(event, closure_ref.unchecked_ref())
            .map_err(Error::from)
    })?;

    Ok(closure)
}

/// Unregisters a mouse event handler from the specified element.
/// The closure is consumed as its lifecycle ends with deregistration.
pub(super) fn unregister_mouse_event_handler(
    element: &Element,
    closure: Closure<dyn FnMut(web_sys::MouseEvent)>,
) -> Result<(), Error> {
    MOUSE_EVENTS.iter().try_for_each(|event| {
        element
            .remove_event_listener_with_callback(event, closure.as_ref().unchecked_ref())
            .map_err(Error::from)
    })
}
