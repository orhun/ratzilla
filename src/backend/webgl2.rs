use crate::{
    backend::{color::to_rgb, utils::*},
    error::Error,
    event::{KeyEvent, MouseEvent},
    render::WebEventHandler,
    CursorShape,
};
use beamterm_renderer::{CellData, FontAtlas, Renderer, TerminalGrid};
use ratatui::{
    backend::WindowSize,
    buffer::Cell,
    layout::{Position, Size},
    prelude::Backend,
    style::{Color, Modifier, Style},
};
use std::{cmp::min, io::Result as IoResult, mem::swap};

// Labels used by the Performance API
const SYNC_TERMINAL_BUFFER_MARK: &str = "sync-terminal-buffer";
const UPLOAD_CELLS_TO_GPU_MARK: &str = "upload-cells-to-gpu";
const WEBGL_RENDER_MARK: &str = "webgl-render";

/// Options for the [`WebGl2Backend`].
#[derive(Debug, Default)]
pub struct WebGl2BackendOptions {
    /// The element ID.
    grid_id: Option<String>,
    /// Size of the render area.
    ///
    /// Overrides the automatically detected size if set.
    size: Option<(u32, u32)>,
    /// Measure performance using the `performance` API.
    measure_performance: bool,
}

impl WebGl2BackendOptions {
    /// Constructs a new [`WebGl2BackendOptions`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the element id of the canvas' parent element.
    pub fn grid_id(mut self, id: &str) -> Self {
        self.grid_id = Some(id.to_string());
        self
    }

    /// Sets the size of the canvas, in pixels.
    pub fn size(mut self, size: (u32, u32)) -> Self {
        self.size = Some(size);
        self
    }

    /// Enables frame-based measurements using the
    /// [Performance](https://developer.mozilla.org/en-US/docs/Web/API/Performance) API.
    pub fn measure_performance(mut self, measure: bool) -> Self {
        self.measure_performance = measure;
        self
    }
}

/// WebGl2 renderer and context.
#[derive(Debug)]
struct WebGl2 {
    /// The WebGL2 renderer.
    renderer: Renderer,
    /// Drawable representation of the terminal
    terminal_grid: TerminalGrid,
}

impl WebGl2 {
    /// Constructs a new [`WebGl2`].
    fn new(parent_element: web_sys::Element, width: u32, height: u32) -> Result<Self, Error> {
        let canvas = create_canvas_in_element(&parent_element, width, height)?;

        let renderer = Renderer::create_with_canvas(canvas)?;
        let terminal_grid = TerminalGrid::new(
            renderer.gl(),
            FontAtlas::load_default(renderer.gl())?,
            renderer.canvas_size(),
        )?;

        Ok(Self {
            terminal_grid,
            renderer,
        })
    }

    fn terminal_size(&self) -> (u16, u16) {
        self.terminal_grid.terminal_size()
    }
}

/// WebGl2 backend for high-performance terminal rendering.
///
/// This backend renders the terminal buffer onto an HTML canvas element using [WebGL2]
/// and the [beamterm renderer].
///
/// [WebGL2]: https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API
/// [beamterm renderer]: https://crates.io/crates/beamterm-renderer
///
/// WebGL2 is supported in all modern browsers (Chrome 56+, Firefox 51+, Safari 15+).
///
/// ## Font Atlas Limitation
///
/// [`WebGl2Backend`] uses prebuilt font atlases for performance. Characters not in the atlas
/// will display as ` `. Use [`CanvasBackend`] if you need dynamic Unicode/emoji support.
///
/// [`CanvasBackend`]: crate::backend::canvas::CanvasBackend
///
/// # Performance Measurement
///
/// The backend supports built-in performance profiling using the browser's Performance API.
/// When enabled via [`WebGl2BackendOptions::measure_performance`], it tracks the duration
/// of each operation:
///
/// | Label                  | Operation                                                   |
/// |------------------------|-------------------------------------------------------------|
/// | `sync-terminal-buffer` | Updating the internal buffer with cell changes from Ratatui |
/// | `upload-cells-to-gpu`  | Uploading changed cell data to GPU buffers                  |
/// | `webgl-render`         | Executing the WebGL draw call to render the terminal        |
///
/// ## Viewing Performance Measurements
///
/// To view the performance measurements in your browser:
///
/// 1. Enable performance measurement when creating the backend
/// 2. Open your browser's Developer Tools (F12 or Ctrl+Shift+I/J)
/// 3. Navigate to the **Performance** tab
/// 4. Collect measurements with the "Record" button, then stop recording
/// 4. Zoom in on a frame and look for the **User Timing** section which will show:
///    - Individual timing marks for each operation
///    - Duration measurements between start and end of each operation
///
/// Alternatively, in the browser console, you can query measurements:
///
/// ```javascript
/// // View all measurements
/// performance.getEntriesByType('measure')
///
/// // View specific operation
/// performance.getEntriesByName('webgl-render')
///
/// // Calculate average time for last 100 measurements
/// const avg = (name) => {
///   const entries = performance.getEntriesByName(name).slice(-100);
///   return entries.reduce((sum, e) => sum + e.duration, 0) / entries.length;
/// };
/// avg('webgl-render')
/// avg('upload-cells-to-gpu')
/// avg('sync-terminal-buffer')
/// ```
#[derive(Debug)]
pub struct WebGl2Backend {
    /// Current buffer.
    buffer: Vec<Cell>,
    /// Indicates if the cells have changed, requiring a
    dirty_cell_data: bool,
    /// WebGl2 context.
    context: WebGl2,
    /// Cursor position.
    cursor_position: Option<Position>,
    /// The cursor shape.
    cursor_shape: CursorShape,
    /// Performance measurement.
    performance: Option<web_sys::Performance>,
}

impl WebGl2Backend {
    /// Constructs a new [`WebGl2Backend`].
    pub fn new() -> Result<Self, Error> {
        let (width, height) = get_raw_window_size();
        Self::new_with_size(width.into(), height.into())
    }

    /// Constructs a new [`WebGl2Backend`] with the given size.
    pub fn new_with_size(width: u32, height: u32) -> Result<Self, Error> {
        Self::new_with_options(WebGl2BackendOptions {
            size: Some((width, height)),
            ..Default::default()
        })
    }

    /// Constructs a new [`WebGl2Backend`] with the given options.
    pub fn new_with_options(options: WebGl2BackendOptions) -> Result<Self, Error> {
        let performance = if options.measure_performance {
            Some(performance()?)
        } else {
            None
        };

        // Parent element of canvas (uses <body> unless specified)
        let parent = get_element_by_id_or_body(options.grid_id.as_ref())?;

        let (width, height) = options
            .size
            .unwrap_or_else(|| (parent.client_width() as u32, parent.client_height() as u32));

        let context = WebGl2::new(parent, width, height)?;
        let buffer = vec![Cell::default(); context.terminal_grid.cell_count()];
        Ok(Self {
            buffer,
            dirty_cell_data: false,
            context,
            cursor_position: None,
            cursor_shape: CursorShape::SteadyBlock,
            performance,
        })
    }

    /// Sets the background color of the canvas.
    ///
    /// TODO: Pass onto the beamterm renderer once it supports it
    pub fn set_background_color(&mut self, _color: Color) {
        unimplemented!()
    }

    /// Returns the [`CursorShape`].
    pub fn cursor_shape(&self) -> &CursorShape {
        &self.cursor_shape
    }

    /// Set the [`CursorShape`].
    pub fn set_cursor_shape(mut self, shape: CursorShape) -> Self {
        self.cursor_shape = shape;
        self
    }

    /// Sets the canvas viewport and projection, reconfigures the terminal grid.
    pub fn resize_canvas(&mut self) -> Result<(), Error> {
        let size_px = self.context.renderer.canvas_size();
        let old_size = self.context.terminal_size();

        // resize the terminal grid and viewport
        let gl = self.context.renderer.gl();
        self.context.terminal_grid.resize(gl, size_px)?;
        self.context.renderer.resize(size_px.0, size_px.1);

        // resize the buffer if needed
        let new_size = self.context.terminal_size();
        if new_size != old_size {
            self.dirty_cell_data = true;

            let cells = &self.buffer;
            self.buffer = resize_cell_grid(cells, old_size, new_size);
        }

        Ok(())
    }

    // Synchronizes the terminal buffer with beamterm's terminal grid.
    fn update_grid(&mut self) -> Result<(), Error> {
        if self.dirty_cell_data {
            self.measure_begin(UPLOAD_CELLS_TO_GPU_MARK);
            let gl = self.context.renderer.gl();
            let terminal = &mut self.context.terminal_grid;
            let cells = self.buffer.iter().map(cell_data);

            terminal.update_cells(gl, cells)?;

            self.dirty_cell_data = false;
            self.measure_end(UPLOAD_CELLS_TO_GPU_MARK);
        }

        Ok(())
    }

    /// Checks if the canvas size matches the display size and resizes it if necessary.
    fn check_canvas_resize(&mut self) -> Result<(), Error> {
        let canvas = self.context.renderer.canvas();
        let display_width = canvas.client_width() as u32;
        let display_height = canvas.client_height() as u32;

        let buffer_width = canvas.width();
        let buffer_height = canvas.height();

        if display_width != buffer_width || display_height != buffer_height {
            canvas.set_width(display_width);
            canvas.set_height(display_height);

            self.resize_canvas()?;
        }

        Ok(())
    }

    /// Draws the cursor at the specified position.
    fn draw_cursor(&mut self, pos: Position) -> IoResult<()> {
        let w = self.context.terminal_size().0 as usize;
        let idx = pos.y as usize * w + pos.x as usize;

        if idx < self.buffer.len() {
            let cursor_style = self.cursor_shape.show(self.buffer[idx].style());
            self.buffer[idx].set_style(cursor_style);
        }

        Ok(())
    }

    /// Measures the beginning of a performance mark.
    fn measure_begin(&self, label: &str) {
        if let Some(performance) = &self.performance {
            performance.mark(label).unwrap_or_default();
        }
    }

    /// Measures the end of a performance mark.
    fn measure_end(&self, label: &str) {
        if let Some(performance) = &self.performance {
            performance
                .measure_with_start_mark(label, label)
                .unwrap_or_default();
        }
    }
}

impl Backend for WebGl2Backend {
    // Populates the buffer with the *updated* cell content.
    fn draw<'a, I>(&mut self, content: I) -> IoResult<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        if content.size_hint().1 == Some(0) {
            // No content to draw, nothing to do
            return Ok(());
        } else {
            // Mark the cell data as dirty; triggers update_grid()
            self.dirty_cell_data = true;
        }

        // Render existing content to the canvas.
        self.measure_begin(WEBGL_RENDER_MARK);
        let terminal = &mut self.context.terminal_grid;
        self.context.renderer.render(terminal);
        self.measure_end(WEBGL_RENDER_MARK);

        // Update internal cell buffer with the new content
        self.measure_begin(SYNC_TERMINAL_BUFFER_MARK);
        let w = self.context.terminal_grid.terminal_size().0 as usize;
        for (x, y, updated_cell) in content {
            let (x, y) = (x as usize, y as usize);
            self.buffer[y * w + x] = cell_with_safe_colors(updated_cell);
        }
        self.measure_end(SYNC_TERMINAL_BUFFER_MARK);

        // Draw the cursor if set
        if let Some(pos) = self.cursor_position {
            self.draw_cursor(pos)?;
        }

        Ok(())
    }

    /// Flush the content to the screen.
    ///
    /// This function is called after the [`WebGl2Backend::draw`] function to
    /// actually render the content to the screen.
    fn flush(&mut self) -> IoResult<()> {
        self.check_canvas_resize()?;
        self.update_grid()?;
        Ok(())
    }

    fn hide_cursor(&mut self) -> IoResult<()> {
        if let Some(pos) = self.cursor_position {
            let y = pos.y as usize;
            let x = pos.x as usize;
            let w = self.context.terminal_grid.terminal_size().0 as usize;

            if let Some(cell) = self.buffer.get_mut(y * w + x) {
                let style = self.cursor_shape.hide(cell.style());
                cell.set_style(style);
            }
        }

        self.cursor_position = None;
        Ok(())
    }

    fn show_cursor(&mut self) -> IoResult<()> {
        Ok(())
    }

    fn clear(&mut self) -> IoResult<()> {
        self.buffer.fill(default_cell());
        Ok(())
    }

    fn size(&self) -> IoResult<Size> {
        let (w, h) = self.context.terminal_grid.terminal_size();
        Ok(Size::new(w as _, h as _))
    }

    fn window_size(&mut self) -> IoResult<WindowSize> {
        let (cols, rows) = self.context.terminal_grid.terminal_size();
        let (w, h) = self.context.renderer.canvas_size();

        Ok(WindowSize {
            columns_rows: Size::new(cols, rows),
            pixels: Size::new(w as _, h as _),
        })
    }

    fn get_cursor_position(&mut self) -> IoResult<Position> {
        match self.cursor_position {
            None => Ok((0, 0).into()),
            Some(position) => Ok(position),
        }
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> IoResult<()> {
        let w = self.context.terminal_grid.terminal_size().0 as usize;

        let new_pos = position.into();
        if let Some(old_pos) = self.cursor_position {
            if old_pos == new_pos {
                return Ok(()); // No change in cursor position
            }

            let y = old_pos.y as usize;
            let x = old_pos.x as usize;

            let old_idx = y * w + x;
            if let Some(old_cell) = self.buffer.get_mut(old_idx) {
                let style = self.cursor_shape.hide(old_cell.style());
                old_cell.set_style(style);
            }
        }
        self.cursor_position = Some(new_pos);
        Ok(())
    }
}

impl WebEventHandler for WebGl2Backend {
    fn setup_mouse_events<F>(&mut self, _callback: F) -> Result<(), Error>
    where
        F: FnMut(MouseEvent) + 'static,
    {
        Err(Error::MouseEventsNotSupported)
    }

    fn clear_mouse_events(&mut self) -> Result<(), Error> {
        // No-op since mouse events are not supported
        Ok(())
    }

    fn setup_key_events<F>(&mut self, mut callback: F) -> Result<(), Error>
    where
        F: FnMut(KeyEvent) + 'static,
    {
        use web_sys::wasm_bindgen::{prelude::Closure, JsCast};
        
        // Note: This implementation doesn't store the closure for cleanup
        // This maintains the same behavior as the original WebRenderer::on_key_event
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
            callback(event.into());
        });
        
        let window = web_sys::window().ok_or(Error::UnableToRetrieveWindow)?;
        let document = window.document().ok_or(Error::UnableToRetrieveDocument)?;
        
        document
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .map_err(Error::from)?;
            
        closure.forget(); // Note: This leaks memory if called multiple times
        Ok(())
    }

    fn clear_key_events(&mut self) -> Result<(), Error> {
        // Cannot clear keys without storing the closure reference
        // This matches the original WebRenderer behavior
        Ok(())
    }
}

/// Resizes the cell grid to the new size, copying existing cells where possible.
///
/// When the terminal dimensions change, this function creates a new cell buffer and
/// preserves existing content in the overlapping region. Any cells outside the overlap
/// are populated with default values.
///
/// # Arguments
/// * `cells` - Current cell buffer to resize
/// * `old_size` - Previous terminal dimensions (cols, rows)
/// * `new_size` - New terminal dimensions (cols, rows)
///
/// # Returns
/// A new cell buffer sized to `new_size`.
fn resize_cell_grid(cells: &[Cell], old_size: (u16, u16), new_size: (u16, u16)) -> Vec<Cell> {
    let old_size = (old_size.0 as usize, old_size.1 as usize);
    let new_size = (new_size.0 as usize, new_size.1 as usize);

    let new_len = new_size.0 * new_size.1;

    let mut new_cells = Vec::with_capacity(new_len);
    for _ in 0..new_len {
        new_cells.push(default_cell());
    }

    // restrict dimensions to the overlapping area
    for y in 0..min(old_size.1, new_size.1) {
        for x in 0..min(old_size.0, new_size.0) {
            // translate x,y to index for old and new buffer
            let new_idx = y * new_size.0 + x;
            let old_idx = y * old_size.0 + x;
            new_cells[new_idx] = cells[old_idx].clone();
        }
    }

    new_cells
}

fn cell_with_safe_colors(cell: &Cell) -> Cell {
    let mut fg = cell.fg;
    let mut bg = cell.bg;

    if cell.modifier.contains(Modifier::REVERSED) {
        swap(&mut fg, &mut bg);
    }

    let mut c = cell.clone();
    c.set_fg(fg);
    c.set_bg(bg);
    c
}

fn default_cell() -> Cell {
    Cell::default()
        .set_style(Style::default().fg(Color::White).bg(Color::Black))
        .clone()
}

fn cell_data(cell: &Cell) -> CellData {
    CellData::new_with_style_bits(
        cell.symbol(),
        into_glyph_bits(cell.modifier),
        to_rgb(cell.fg, 0xffffff),
        to_rgb(cell.bg, 0x000000),
    )
}

/// Extracts glyph styling bits from cell modifiers.
///
/// # Performance Optimization
/// Bitwise operations are used instead of individual `contains()` checks.
/// This provides a ~50% performance improvement over the naive approach.
///
/// # Bit Layout Reference
///
/// ```plain
/// Modifier bits:     0000_0000_0000_0001  (BOLD at bit 0)
///                    0000_0000_0000_0100  (ITALIC at bit 2)
///                    0000_0000_0000_1000  (UNDERLINED at bit 3)
///                    0000_0001_0000_0000  (CROSSED_OUT at bit 8)
///
/// FontStyle bits:    0000_0010_0000_0000  (Bold as bit 9)
///                    0000_0100_0000_0000  (Italic as bit 10)
/// GlyphEffect bits:  0001_0000_0000_0000  (Underline at bit 12)
///                    0010_0000_0000_0000  (Strikethrough at bit 13)
///
/// Shift operations:  bit 0 << 9 = bit 9
///                    bit 2 << 8 = bit 10
///                    bit 3 << 9 = bit 12
///                    bit 8 << 5 = bit 13
/// ```
const fn into_glyph_bits(modifier: Modifier) -> u16 {
    let m = modifier.bits();

    (m << 9) & (1 << 9)    // bold
    | (m << 8) & (1 << 10) // italic
    | (m << 9) & (1 << 12) // underline
    | (m << 5) & (1 << 13) // strikethrough
}

#[cfg(test)]
mod tests {
    use super::*;
    use beamterm_renderer::{FontStyle, GlyphEffect};
    use ratatui::style::Modifier;

    #[test]
    fn test_font_style() {
        [
            (FontStyle::Bold, Modifier::BOLD),
            (FontStyle::Italic, Modifier::ITALIC),
            (FontStyle::BoldItalic, Modifier::BOLD | Modifier::ITALIC),
        ]
        .into_iter()
        .map(|(style, modifier)| (style as u16, into_glyph_bits(modifier)))
        .for_each(|(expected, actual)| assert_eq!(expected, actual));
    }

    #[test]
    fn test_glyph_effect() {
        [
            (GlyphEffect::Underline, Modifier::UNDERLINED),
            (GlyphEffect::Strikethrough, Modifier::CROSSED_OUT),
        ]
        .into_iter()
        .map(|(effect, modifier)| (effect as u16, into_glyph_bits(modifier)))
        .for_each(|(expected, actual)| assert_eq!(expected, actual));
    }
}
