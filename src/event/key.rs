//! Module for `KeyEvent` and related structs.

use bitflags::bitflags;

/// A key event.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KeyEvent {
    /// The key code.
    pub code: KeyCode,
    /// Additional key modifiers.
    pub modifiers: KeyModifiers,
    /// Kind of event.
    pub kind: KeyEventKind,
    /// Keyboard state.
    pub state: KeyEventState,
}

/// Represents a keyboard event kind.
#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
pub enum KeyEventKind {
    /// A key has been pressed.
    ///
    /// **Note:** this correlates to `keydown`, not `keypress`.
    Press,
    /// Any event in which `event.repeat` is true.
    /// This is mostly kept for parity.
    Repeat,
    /// A key has been released.
    Release,
}

bitflags! {
    /// Represents key modifiers (shift, control, alt, etc.).
    #[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
    pub struct KeyModifiers: u8 {
        /// Whether the shift key is pressed.
        const SHIFT = 0b0000_0001;
        /// Whether the control key is pressed.
        const CONTROL = 0b0000_0010;
        /// Whether the alt key is pressed.
        const ALT = 0b0000_0100;
        /// Whether the meta key is pressed.
        const META = 0b0010_0000;
        /// No key is pressed.
        const NONE = 0b0000_0000;
    }
}

bitflags! {
    /// Represents extra state about the key event.
    #[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Hash)]
    pub struct KeyEventState: u8 {
        /// The key event origins from the keypad.
        const KEYPAD = 0b0000_0001;
        /// Caps Lock was enabled for this key event.
        ///
        /// **Note:** this is set for the initial press of Caps Lock itself.
        const CAPS_LOCK = 0b0000_0010;
        /// Num Lock was enabled for this key event.
        ///
        /// **Note:** this is set for the initial press of Num Lock itself.
        const NUM_LOCK = 0b0000_0100;
        /// No other state applied.
        const NONE = 0b0000_0000;
    }
}

/// Convert a [`web_sys::KeyboardEvent`] to a [`KeyEvent`].
impl From<web_sys::KeyboardEvent> for KeyEvent {
    fn from(event: web_sys::KeyboardEvent) -> Self {
        let shift = if event.shift_key() {
            KeyModifiers::SHIFT
        } else {
            KeyModifiers::NONE
        };
        let ctrl = if event.ctrl_key() {
            KeyModifiers::CONTROL
        } else {
            KeyModifiers::NONE
        };
        let alt = if event.alt_key() {
            KeyModifiers::ALT
        } else {
            KeyModifiers::NONE
        };
        KeyEvent {
            code: event.into(),
            modifiers: shift | ctrl | alt,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
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
        let code = event.code();
        let key = event.key();
        if key.len() == 1 {
            if let Some(char) = key.chars().next() {
                return KeyCode::Char(char);
            } else {
                return KeyCode::Unidentified;
            }
        }
        match code.as_str() {
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
