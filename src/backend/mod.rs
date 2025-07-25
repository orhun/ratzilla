//! ## Backends
//!
//! **Ratzilla** provides three backends for rendering terminal UIs in the browser,
//! each with different performance characteristics and trade-offs:
//!
//! - [`WebGl2Backend`]: GPU-accelerated rendering powered by [beamterm][beamterm]. Uses prebuilt
//!   font atlases. Best performance, capable of 60fps on large terminals (300x100+).
//!
//! - [`CanvasBackend`]: Canvas 2D API with full Unicode support via browser font rendering.
//!   Good fallback when WebGL2 isn't available or when dynamic character support is required.
//!   Does not support hyperlinks or text selection, but can render dynamic Unicode/emoji.
//!
//! - [`DomBackend`]: Renders cells as HTML elements. Most compatible and accessible,
//!   supports hyperlinks, but slowest for large terminals.
//!
//! [beamterm]: https://github.com/junkdog/beamterm
//!
//! ## Backend Comparison
//!
//! | Feature                      | DomBackend | CanvasBackend | WebGl2Backend    |
//! |------------------------------|------------|---------------|------------------|
//! | **60fps on large terminals** | ✗          | ✗             | ✓                |
//! | **Memory Usage**             | Highest    | Medium        | Lowest           |
//! | **Hyperlinks**               | ✓          | ✗             | ✓                |
//! | **Text Selection**           | ✓          | ✗             | ✓                |
//! | **Accessibility**            | ✓          | Limited       | Limited          |
//! | **Unicode/Emoji Support**    | Full       | Full          | Limited to atlas |
//! | **Dynamic Characters**       | ✓          | ✓             | ✗                |
//! | **Font Variants**            | ✓          | Regular only  | ✓                |
//! | **Underline**                | ✓          | ✗             | ✓                |
//! | **Strikethrough**            | ✓          | ✗             | ✓                |
//! | **Browser Support**          | All        | All           | Modern (2017+)   |
//!
//! ## Choosing a Backend
//!
//! - **WebGl2Backend**: Preferred for most applications - consumes the least amount of resources
//! - **CanvasBackend**: When you need dynamic Unicode/emoji or must support non-WebGL2 browsers
//! - **DomBackend**: When you need better accessibility or CSS styling

/// Canvas backend.
pub mod canvas;

/// DOM backend.
pub mod dom;

/// WebGL2 backend.
pub mod webgl2;

/// Color handling.
mod color;
/// Backend utilities.
pub(crate) mod utils;

/// Cursor shapes.
pub mod cursor;

/// Re-export beamterm's atlas data type. Used by [`WebGl2BackendOptions::font_atlas`].
pub use beamterm_renderer::FontAtlasData;
