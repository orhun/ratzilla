use crate::{
    backend::utils::get_document,
    error::Error,
    event::{KeyEvent, MouseEvent, MouseEventKind},
};
use std::{cell::RefCell, rc::Rc};
use web_sys::{
    wasm_bindgen::{closure::Closure, JsCast},
    Element, HtmlElement,
};

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

/// Manages web event listeners with automatic cleanup.
///
/// This struct wraps JavaScript event listeners with proper lifecycle management,
/// automatically removing event listeners when the struct is dropped.
#[derive(Debug)]
pub(super) struct EventCallback<T> {
    event_types: &'static [&'static str],
    element: Element,
    closure: Closure<dyn FnMut(T)>,
}

impl EventCallback<web_sys::KeyboardEvent> {
    /// Creates a new keyboard event callback that listens for keydown events.
    ///
    /// # Arguments
    /// * `element` - The DOM element to store (though events are registered on document)
    /// * `callback` - Function to call when keyboard events occur
    pub fn new_key<F>(element: Element, mut callback: F) -> Result<Self, Error>
    where
        F: FnMut(KeyEvent) + 'static,
    {
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
            callback(event.into());
        });

        get_document()?
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .map_err(Error::from)?;

        Ok(Self {
            event_types: &["keydown"],
            element,
            closure,
        })
    }
}

impl EventCallback<web_sys::MouseEvent> {
    /// Creates a new mouse event callback with coordinate transformation.
    ///
    /// Registers listeners for all mouse events (move, down, up, enter, leave, click, dblclick, wheel)
    /// and converts browser coordinates to terminal grid coordinates.
    ///
    /// # Arguments
    /// * `element` - The DOM element to listen on
    /// * `grid_width` - Terminal width in characters for coordinate mapping
    /// * `grid_height` - Terminal height in characters for coordinate mapping  
    /// * `offset` - Optional pixel offset for coordinate calculation (e.g., canvas padding)
    /// * `callback` - Function to call when mouse events occur
    pub fn new_mouse<F>(
        element: Element,
        grid_width: u16,
        grid_height: u16,
        offset: Option<f64>,
        callback: F,
    ) -> Result<Self, Error>
    where
        F: FnMut(MouseEvent) + 'static,
    {
        let closure = register_mouse_event_handler_with_wheel_normalization(
            &element,
            grid_width,
            grid_height,
            offset,
            callback,
        )?;

        Ok(Self {
            event_types: MOUSE_EVENTS,
            element,
            closure,
        })
    }
}

/// Converts mouse coordinates to grid coordinates using element dimensions
/// This is the core function both backends use for accurate coordinate calculation
fn mouse_to_grid_coords(
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
fn register_mouse_event_handler(
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

/// Registers a mouse event handler that normalizes wheel deltas to sensible terminal scroll amounts.
fn register_mouse_event_handler_with_wheel_normalization<F>(
    element: &Element,
    grid_width: u16,
    grid_height: u16,
    offset: Option<f64>,
    callback: F,
) -> Result<Closure<dyn FnMut(web_sys::MouseEvent)>, Error>
where
    F: FnMut(MouseEvent) + 'static,
{
    let callback = Rc::new(RefCell::new(callback));
    let element_clone = element.clone();

    let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        if let Some(html_element) = element_clone.dyn_ref::<web_sys::HtmlElement>() {
            let mut mouse_event =
                mouse_to_grid_coords(&event, html_element, grid_width, grid_height, offset);

            // Normalize wheel deltas if it's a wheel event
            if let MouseEventKind::Wheel {
                delta_col,
                delta_row,
            } = mouse_event.kind
            {
                if let Ok(wheel_event) = event.dyn_into::<web_sys::WheelEvent>() {
                    let normalized_deltas = normalize_wheel_deltas(
                        wheel_event.delta_mode(),
                        delta_col as f64,
                        delta_row as f64,
                    );

                    mouse_event.kind = MouseEventKind::Wheel {
                        delta_col: normalized_deltas.0 as i16,
                        delta_row: normalized_deltas.1 as i16,
                    };
                }
            }

            callback.borrow_mut()(mouse_event);
        }
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);

    register_mouse_event_handler(element, closure)
}

/// Normalizes wheel deltas to sensible terminal scroll amounts (max 3 lines per tick).
fn normalize_wheel_deltas(delta_mode: u32, delta_col: f64, delta_row: f64) -> (f64, f64) {
    fn normalize_single_delta(delta: f64, delta_mode: u32) -> f64 {
        if delta == 0.0 {
            return 0.0;
        }

        let sign = delta.signum();

        // 0: DOM_DELTA_PIXEL - convert to 1-3 lines based on magnitude
        // 1: DOM_DELTA_LINE  - clamp to max 3 lines
        // 2: DOM_DELTA_PAGE  - treat as 3 lines
        (sign * match delta_mode {
            0 => (delta.abs() / 50.0) - 25.0,
            1 => delta,
            2 => 3.0,
            _ => 0.0,
        }).clamp(-3.0, 3.0)
    }

    (
        normalize_single_delta(delta_col, delta_mode),
        normalize_single_delta(delta_row, delta_mode),
    )
}

impl<T> Drop for EventCallback<T> {
    fn drop(&mut self) {
        let closure = &self.closure.as_ref();
        for event_type in self.event_types {
            let _ = self
                .element
                .remove_event_listener_with_callback(event_type, closure.unchecked_ref());
        }
    }
}
