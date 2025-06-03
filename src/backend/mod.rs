//! ## Backends
//!
//! **Ratzilla** currently supports the following backends:
//!
//! 1. [`DomBackend`]: Works by converting the cells to HTML elements (e.g. `<span>`, `<a>`, etc.) and rendering them in the DOM.
//! 2. [`CanvasBackend`]: Works by rendering the cells in a HTML canvas element.
//!
//! ## Comparison
//!
//! The [`DomBackend`] is more flexible and easier to style, but it can be slower for large TUIs. The [`CanvasBackend`] is faster and more efficient, but does not support all the features of the [`DomBackend`] such as hyperlinks.

/// Canvas backend.
pub mod canvas;

/// DOM backend.
pub mod dom;

/// WebGL2 backend.
pub mod webgl2;

/// Backend utilities.
pub(crate) mod utils;

/// Cursor shapes.
pub mod cursor;
mod elements;
