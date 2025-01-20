#![doc = include_str!("../README.md")]

pub mod error;
pub mod event;
pub mod utils;
pub mod widgets;

mod backend;
mod render;

pub use backend::canvas::CanvasBackend;
pub use backend::dom::DomBackend;
pub use render::RenderOnWeb;

// Re-export ratatui crate.
pub use ratatui;
