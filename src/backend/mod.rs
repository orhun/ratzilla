//! ## Backends
//!
//! **Ratzilla** provides three backends for rendering terminal UIs in the browser,
//! each with different performance characteristics and trade-offs:
//!
//! - [`WebGl2Backend`]: GPU-accelerated rendering with prebuilt font atlases. Best performance,
//!   capable of 60fps on large terminals (300x200+). **This is the recommended backend for most
//!   applications.**
//!
//! - [`CanvasBackend`]: Canvas 2D API with full Unicode support via browser font rendering.
//!   Good fallback when WebGL2 isn't available or when dynamic character support is required.
//!
//! - [`DomBackend`]: Renders cells as HTML elements. Most compatible and accessible,
//!   supports hyperlinks, but slowest for large terminals.
//!
//! ## Backend Comparison
//!
//! | Feature                      | DomBackend | CanvasBackend | WebGl2Backend    |
//! |------------------------------|------------|---------------|------------------|
//! | **60fps on large terminals** | ✗          | ✗             | ✓                |
//! | **Memory Usage**             | Highest    | Medium        | Lowest           |
//! | **Hyperlinks**               | ✓          | ✗             | ✗                |
//! | **Text Selection**           | ✓          | ✗             | ✗                |
//! | **Accessibility**            | ✓          | Limited       | Limited          |
//! | **Unicode/Emoji Support**    | Full       | Full          | Limited to atlas |
//! | **Dynamic Characters**       | ✓          | ✓             | ✗                |
//! | **Browser Support**          | All        | All           | Modern (2017+)   |
//!
//! ## Choosing a Backend
//!
//! - **WebGl2Backend**: Preferred for most applications - consumes the least amount of resources
//! - **CanvasBackend**: When you need dynamic Unicode/emoji or must support non-WebGL2 browsers
//! - **DomBackend**: When you need hyperlinks, accessibility, or CSS styling
//!
//! ## Font Atlas Limitation
//!
//! WebGl2Backend uses prebuilt font atlases for performance. Characters not in the atlas
//! will display as ' '. Use [`CanvasBackend`] if you need dynamic Unicode/emoji support.
//!
//! Note: WebGL2 is supported in all modern browsers (Chrome 56+, Firefox 51+, Safari 15+).

/// Canvas backend.
pub mod canvas;

/// DOM backend.
pub mod dom;

/// WebGL2 backend.
pub mod webgl2;

/// Backend utilities.
pub(crate) mod utils;
/// Color handling.
mod color;

/// Cursor shapes.
pub mod cursor;
