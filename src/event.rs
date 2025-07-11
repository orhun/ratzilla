/// A key event.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KeyEvent {
    /// The key code.
    pub code: KeyCode,
    /// Whether the control key is pressed.
    pub ctrl: bool,
    /// Whether the alt key is pressed.
    pub alt: bool,
    /// Whether the shift key is pressed.
    pub shift: bool,
}

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

/// Convert a [`web_sys::KeyboardEvent`] to a [`KeyEvent`].
impl From<web_sys::KeyboardEvent> for KeyEvent {
    fn from(event: web_sys::KeyboardEvent) -> Self {
        let ctrl = event.ctrl_key();
        let alt = event.alt_key();
        let shift = event.shift_key();
        KeyEvent {
            code: event.into(),
            ctrl,
            alt,
            shift,
        }
    }
}

/// A key code.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum KeyCode {
    /// Normal letter key input.
    Char(char),
    /// F keys.
    F(u8),
    /// Backspace key
    Backspace,
    /// Enter or return key
    Enter,
    /// Left arrow key
    Left,
    /// Right arrow key
    Right,
    /// Up arrow key
    Up,
    /// Down arrow key
    Down,
    /// Tab key
    Tab,
    /// Delete key
    Delete,
    /// Home key
    Home,
    /// End key
    End,
    /// Page up key
    PageUp,
    /// Page down key
    PageDown,
    /// Escape key
    Esc,
    /// Unidentified.
    Unidentified,
}

/// Convert a [`web_sys::KeyboardEvent`] to a [`KeyCode`].
impl From<web_sys::KeyboardEvent> for KeyCode {
    fn from(event: web_sys::KeyboardEvent) -> Self {
        let key = event.key();
        if key.len() == 1 {
            let char = key.chars().next();
            if let Some(char) = char {
                return KeyCode::Char(char);
            } else {
                return KeyCode::Unidentified;
            }
        }
        match key.as_str() {
            "F1" => KeyCode::F(1),
            "F2" => KeyCode::F(2),
            "F3" => KeyCode::F(3),
            "F4" => KeyCode::F(4),
            "F5" => KeyCode::F(5),
            "F6" => KeyCode::F(6),
            "F7" => KeyCode::F(7),
            "F8" => KeyCode::F(8),
            "F9" => KeyCode::F(9),
            "F10" => KeyCode::F(10),
            "F11" => KeyCode::F(11),
            "F12" => KeyCode::F(12),
            "Backspace" => KeyCode::Backspace,
            "Enter" => KeyCode::Enter,
            "ArrowLeft" => KeyCode::Left,
            "ArrowRight" => KeyCode::Right,
            "ArrowUp" => KeyCode::Up,
            "ArrowDown" => KeyCode::Down,
            "Tab" => KeyCode::Tab,
            "Delete" => KeyCode::Delete,
            "Home" => KeyCode::Home,
            "End" => KeyCode::End,
            "PageUp" => KeyCode::PageUp,
            "PageDown" => KeyCode::PageDown,
            "Escape" => KeyCode::Esc,
            _ => KeyCode::Unidentified,
        }
    }
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
