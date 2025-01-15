#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

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
