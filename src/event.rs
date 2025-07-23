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
    /// The triggered event.
    pub kind: MouseEventKind,
    /// The x grid coordinate of the mouse.
    pub col: u16,
    /// The y grid coordinate of the mouse.
    pub row: u16,
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
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
    /// Unnamed mouse button
    Other(i32),
    /// Either left mouse button or no button during move events
    Unidentified,
}

/// A mouse event.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MouseEventKind {
    /// Mouse moved
    Moved,
    /// Mouse button clicked
    ButtonDown(MouseButton),
    /// Mouse button released
    ButtonUp(MouseButton),
    /// Mouse entered element
    Entered,
    /// Mouse left element
    Exited,
    /// Mouse single click (distinct from mousedown)
    SingleClick(MouseButton),
    /// Mouse double click
    DoubleClick(MouseButton),
    /// Mouse wheel scrolled
    Wheel {
        /// Horizontal scroll delta
        delta_x: i16,
        /// Vertical scroll delta
        delta_y: i16,
        /// Z-axis scroll delta
        delta_z: i16,
    },
    /// Unidentified mouse event
    Unidentified,
}

/// Convert a [`web_sys::MouseEvent`] to a [`MouseEvent`].
impl MouseEvent {
    /// Creates a new [`MouseEvent`] from a web mouse event and cell size information.
    ///
    /// This uses viewport-relative coordinates.
    ///
    /// # Arguments
    /// * `event` - The web mouse event from the browser
    /// * `cell_size_px` - The pixel dimensions of a terminal cell (width, height)
    pub fn new(event: web_sys::MouseEvent, cell_size_px: (u32, u32)) -> Self {
        let ctrl = event.ctrl_key();
        let alt = event.alt_key();
        let shift = event.shift_key();
        let event_type = MouseEventKind::from(&event);
        let (col, row) = Self::pixels_to_grid_coords(
            event.client_x() as u32,
            event.client_y() as u32,
            cell_size_px,
        );

        MouseEvent {
            kind: event_type,
            col,
            row,
            ctrl,
            alt,
            shift,
        }
    }

    /// Creates a new [`MouseEvent`] from a web mouse event with coordinates relative to a grid element.
    ///
    /// This calculates mouse coordinates relative to the specified grid element's bounding rectangle.
    ///
    /// # Arguments
    /// * `event` - The web mouse event from the browser
    /// * `cell_size_px` - The pixel dimensions of a terminal cell (width, height)
    /// * `grid_rect` - The bounding rectangle of the grid element (left, top, width, height)
    pub fn new_relative(
        event: web_sys::MouseEvent,
        cell_size_px: (u32, u32),
        grid_rect: (f64, f64, f64, f64),
    ) -> Self {
        debug_assert!(event.x() <= 0xffff);
        debug_assert!(event.y() <= 0xffff);

        let ctrl = event.ctrl_key();
        let alt = event.alt_key();
        let shift = event.shift_key();
        let event_type = MouseEventKind::from(&event);

        // Calculate mouse position relative to the grid element
        let (left, top, _width, _height) = grid_rect;
        let relative_x = event.client_x() as f64 - left;
        let relative_y = event.client_y() as f64 - top;
        let mouse_x = relative_x.max(0.0) as u32;
        let mouse_y = relative_y.max(0.0) as u32;
        let (col, row) = Self::pixels_to_grid_coords(mouse_x, mouse_y, cell_size_px);

        MouseEvent {
            kind: event_type,
            col,
            row,
            ctrl,
            alt,
            shift,
        }
    }

    /// Converts pixel coordinates to grid coordinates.
    fn pixels_to_grid_coords(pixel_x: u32, pixel_y: u32, cell_size_px: (u32, u32)) -> (u16, u16) {
        let col = (pixel_x / cell_size_px.0) as u16;
        let row = (pixel_y / cell_size_px.1) as u16;
        (col, row)
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
impl From<&web_sys::MouseEvent> for MouseEventKind {
    fn from(event: &web_sys::MouseEvent) -> Self {
        use web_sys::wasm_bindgen::JsCast;

        let event_type = event.type_();
        match event_type.as_str() {
            "mousemove" => MouseEventKind::Moved,
            "mousedown" => MouseEventKind::ButtonDown(event.button().into()),
            "mouseup" => MouseEventKind::ButtonUp(event.button().into()),
            "mouseenter" => MouseEventKind::Entered,
            "mouseleave" => MouseEventKind::Exited,
            "click" => MouseEventKind::SingleClick(event.button().into()),
            "dblclick" => MouseEventKind::DoubleClick(event.button().into()),
            "wheel" => {
                if let Ok(wheel_event) = event.clone().dyn_into::<web_sys::WheelEvent>() {
                    MouseEventKind::Wheel {
                        delta_x: wheel_event.delta_x() as i16,
                        delta_y: wheel_event.delta_y() as i16,
                        delta_z: wheel_event.delta_z() as i16,
                    }
                } else {
                    MouseEventKind::Wheel {
                        delta_x: 0,
                        delta_y: 0,
                        delta_z: 0,
                    }
                }
            }
            _ => MouseEventKind::Unidentified,
        }
    }
}
