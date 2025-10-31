//! Module for `MouseEvent` and related structs.

/// A mouse movement event.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MouseEvent {
    /// The mouse button that was pressed.
    pub button: MouseButton,
    /// The triggered event.
    pub event: MouseEventKind,
    /// The x coordinate of the mouse.
    pub x: u32,
    /// The y coordinate of the mouse.
    pub y: u32,
    /// Whether the control key is pressed.
    pub ctrl: bool,
    /// Whether the alt key is pressed.
    pub alt: bool,
    /// Whether the shift key is pressed.
    pub shift: bool,
}

/// A mouse button.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MouseButton {
    /// Left mouse button
    Left,
    /// Right mouse button
    Right,
    /// Middle mouse button
    Middle,
    /// Back mouse button
    Back,
    /// Forward mouse button
    Forward,
    /// Unidentified mouse button
    Unidentified,
}

/// A mouse event.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MouseEventKind {
    /// Mouse moved
    Moved,
    /// Mouse button pressed
    Pressed,
    /// Mouse button released
    Released,
    /// Unidentified mouse event
    Unidentified,
}

/// Convert a [`web_sys::MouseEvent`] to a [`MouseEvent`].
impl From<web_sys::MouseEvent> for MouseEvent {
    fn from(event: web_sys::MouseEvent) -> Self {
        let ctrl = event.ctrl_key();
        let alt = event.alt_key();
        let shift = event.shift_key();
        let event_type = event.type_().into();
        MouseEvent {
            // Button is only valid if it is a mousedown or mouseup event.
            button: if event_type == MouseEventKind::Moved {
                MouseButton::Unidentified
            } else {
                event.button().into()
            },
            event: event_type,
            x: event.client_x() as u32,
            y: event.client_y() as u32,
            ctrl,
            alt,
            shift,
        }
    }
}

/// Convert a [`web_sys::MouseEvent`] to a [`MouseButton`].
impl From<i16> for MouseButton {
    fn from(button: i16) -> Self {
        match button {
            0 => MouseButton::Left,
            1 => MouseButton::Middle,
            2 => MouseButton::Right,
            3 => MouseButton::Back,
            4 => MouseButton::Forward,
            _ => MouseButton::Unidentified,
        }
    }
}

/// Convert a [`web_sys::MouseEvent`] to a [`MouseEventKind`].
impl From<String> for MouseEventKind {
    fn from(event: String) -> Self {
        let event = event.as_str();
        match event {
            "mousemove" => MouseEventKind::Moved,
            "mousedown" => MouseEventKind::Pressed,
            "mouseup" => MouseEventKind::Released,
            _ => MouseEventKind::Unidentified,
        }
    }
}