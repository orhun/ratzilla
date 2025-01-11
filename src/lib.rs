mod backend;
mod canvas_backend;
mod canvas_utils;
mod render;
mod render_canvas;
pub mod utils;

pub mod widgets;

pub use backend::WasmBackend;
pub use canvas_backend::WasmCanvasBackend;
pub use render::RenderOnWeb;
pub use render_canvas::RenderOnWebCanvas;

// Re-export ratatui crate.
pub use ratatui;
