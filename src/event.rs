#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KeyEvent {
    pub key: Key,
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
            key: event.into(),
            ctrl,
            alt,
            shift,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Key {
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

impl From<web_sys::KeyboardEvent> for Key {
    fn from(event: web_sys::KeyboardEvent) -> Self {
        let key = event.key();
        if key.len() == 1 {
            let char = key.chars().next();
            if let Some(char) = char {
                return Key::Char(char);
            } else {
                return Key::Unidentified;
            }
        }
        match key.as_str() {
            "F1" => Key::F(1),
            "F2" => Key::F(2),
            "F3" => Key::F(3),
            "F4" => Key::F(4),
            "F5" => Key::F(5),
            "F6" => Key::F(6),
            "F7" => Key::F(7),
            "F8" => Key::F(8),
            "F9" => Key::F(9),
            "F10" => Key::F(10),
            "F11" => Key::F(11),
            "F12" => Key::F(12),
            "Backspace" => Key::Backspace,
            "Enter" => Key::Enter,
            "ArrowLeft" => Key::Left,
            "ArrowRight" => Key::Right,
            "ArrowUp" => Key::Up,
            "ArrowDown" => Key::Down,
            "Tab" => Key::Tab,
            "Delete" => Key::Delete,
            "Home" => Key::Home,
            "End" => Key::End,
            "PageUp" => Key::PageUp,
            "PageDown" => Key::PageDown,
            "Escape" => Key::Esc,
            _ => Key::Unidentified,
        }
    }
}
