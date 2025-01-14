mod backend;
mod render;
pub mod utils;

pub mod widgets;

pub use backend::canvas::CanvasBackend;
pub use backend::dom::DomBackend;
pub use render::RenderOnWeb;

// Re-export ratatui crate.
pub use ratatui;
