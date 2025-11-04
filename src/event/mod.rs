pub mod key;
pub mod mouse;

pub use key::KeyEvent;
pub use mouse::MouseEvent;

/// A generic event.
#[non_exhaustive]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Event {
    /// The terminal gained focus.
    FocusGained,
    /// The terminal lost focus.
    FocusLost,
    /// A single key event with additional pressed modifiers.
    Key(KeyEvent),
    /// A single mouse event with additional pressed modifiers.
    Mouse(MouseEvent),
    /// A string that was pasted into the terminal.
    Paste(String),
    /// An resize event with new dimensions after resize (columns, rows).
    Resize(u16, u16),
}
