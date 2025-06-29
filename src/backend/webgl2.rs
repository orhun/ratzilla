use crate::{
    backend::{color::to_rgb, utils::*},
    error::Error,
    CursorShape,
};
use beamterm_renderer::{
    CellData, FontAtlasData, GlyphEffect, SelectionMode, Terminal as Beamterm,
    TerminalMouseHandler, TerminalMouseEvent, MouseEventType, select,
};
use compact_str::CompactString;
use ratatui::{
    backend::WindowSize,
    buffer::Cell,
    layout::{Position, Size},
    prelude::Backend,
    style::{Color, Modifier},
};
use std::{io::Result as IoResult, mem::swap, rc::Rc, cell::RefCell};
use bitvec::prelude::BitVec;
use crate::widgets::hyperlink::HYPERLINK_MODIFIER;

// Labels used by the Performance API
const SYNC_TERMINAL_BUFFER_MARK: &str = "sync-terminal-buffer";
const WEBGL_RENDER_MARK: &str = "webgl-render";

/// Options for the [`WebGl2Backend`].
#[derive(Default)]
pub struct WebGl2BackendOptions {
    /// The element ID.
    grid_id: Option<String>,
    /// Size of the render area.
    ///
    /// Overrides the automatically detected size if set.
    size: Option<(u32, u32)>,
    /// Fallback glyph to use for characters not in the font atlas.
    fallback_glyph: Option<CompactString>,
    /// Override the default font atlas.
    font_atlas: Option<FontAtlasData>,
    /// The canvas padding color.
    canvas_padding_color: Option<Color>,
    /// The cursor shape.
    cursor_shape: CursorShape,
    /// Whether to use beamterm's internal mouse handler for selection.
    default_mouse_handler: bool,
    /// Enable hyperlinks in the canvas.
    enable_hyperlinks: bool,
    /// Measure performance using the `performance` API.
    measure_performance: bool,
    /// Hyperlink click callback.
    hyperlink_callback: Option<Rc<RefCell<dyn FnMut(&str)>>>,
}

impl WebGl2BackendOptions {
    /// Constructs a new [`WebGl2BackendOptions`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the element id of the canvas' parent element.
    pub fn grid_id(mut self, id: &str) -> Self {
        self.grid_id = Some(id.into());
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

    /// Sets the fallback glyph to use for characters not in the font atlas.
    /// If not set, defaults to a space character (` `).
    pub fn fallback_glyph(mut self, glyph: &str) -> Self {
        self.fallback_glyph = Some(glyph.into());
        self
    }

    /// Sets the canvas padding color. The padding area is the space not covered by the
    /// terminal grid.
    pub fn canvas_padding_color(mut self, color: Color) -> Self {
        self.canvas_padding_color = Some(color);
        self
    }

    /// Sets the cursor shape to use when cursor is visible.
    pub fn cursor_shape(mut self, shape: CursorShape) -> Self {
        self.cursor_shape = shape;
        self
    }

    /// Sets a custom font atlas to use for rendering.
    pub fn font_atlas(mut self, atlas: FontAtlasData) -> Self {
        self.font_atlas = Some(atlas);
        self
    }

    /// Enables block-based mouse selection with automatic copy to
    /// clipboard on selection.
    pub fn enable_mouse_selection(mut self) -> Self {
        self.default_mouse_handler = true;
        self
    }

    /// Enables hyperlinks in the canvas.
    pub fn enable_hyperlinks(mut self) -> Self {
        self.enable_hyperlinks = true;
        self
    }

    /// Sets a callback for when hyperlinks are clicked
    pub fn on_hyperlink_click<F>(mut self, callback: F) -> Self 
    where F: FnMut(&str) + 'static
    {
        self.enable_hyperlinks = true;
        self.hyperlink_callback = Some(Rc::new(RefCell::new(callback)));
        self
    }

    /// Gets the canvas padding color, defaulting to black if not set.
    fn get_canvas_padding_color(&self) -> u32 {
        self.canvas_padding_color
            .map(|c| to_rgb(c, 0x000000))
            .unwrap_or(0x000000)
    }
}

impl std::fmt::Debug for WebGl2BackendOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebGl2BackendOptions")
            .field("grid_id", &self.grid_id)
            .field("size", &self.size)
            .field("fallback_glyph", &self.fallback_glyph)
            .field("canvas_padding_color", &self.canvas_padding_color)
            .field("cursor_shape", &self.cursor_shape)
            .field("default_mouse_handler", &self.default_mouse_handler)
            .field("enable_hyperlinks", &self.enable_hyperlinks)
            .field("measure_performance", &self.measure_performance)
            .field("hyperlink_callback", &"<callback>")
            .finish()
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
/// | `sync-terminal-buffer` | Synchronizes Ratatui's cell data with beamterm's            |
/// | `webgl-render`         | Flushes the GPU buffers and executes the WebGL draw call    |
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
pub struct WebGl2Backend {
    /// WebGl2 terminal renderer.
    beamterm: Beamterm,
    /// The options used to create this backend.
    options: WebGl2BackendOptions,
    /// Cursor position.
    cursor_position: Option<Position>,
    /// Performance measurement.
    performance: Option<web_sys::Performance>,
    /// Hyperlink tracking.
    hyperlink_cells: Option<BitVec>,
    /// Mouse handler for hyperlink clicks.
    hyperlink_mouse_handler: Option<TerminalMouseHandler>,
    /// Hyperlink click callback.
    hyperlink_callback: Option<Rc<RefCell<dyn FnMut(&str)>>>,
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
    pub fn new_with_options(mut options: WebGl2BackendOptions) -> Result<Self, Error> {
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

        let canvas = create_canvas_in_element(&parent, width, height)?;

        let context = Beamterm::builder(canvas)
            .canvas_padding_color(options.get_canvas_padding_color())
            .fallback_glyph(&options.fallback_glyph.as_ref().unwrap_or(&" ".into()))
            .font_atlas(options.font_atlas.take().unwrap_or_default());

        let context = if options.default_mouse_handler {
            context.default_mouse_input_handler(SelectionMode::Block, true)
        } else {
            context
        }.build()?;

        let hyperlink_cells = if options.enable_hyperlinks {
            Some(BitVec::repeat(false, context.cell_count()))
        } else {
            None
        };

        // Extract hyperlink callback from options
        let hyperlink_callback = options.hyperlink_callback.take();

        // Set up hyperlink mouse handler if callback is provided
        let hyperlink_mouse_handler = if let Some(ref callback) = hyperlink_callback {
            Some(Self::create_hyperlink_mouse_handler(&context, hyperlink_cells.as_ref(), callback.clone())?)
        } else {
            None
        };

        Ok(Self {
            beamterm: context,
            cursor_position: None,
            options,
            hyperlink_cells,
            hyperlink_mouse_handler,
            hyperlink_callback,
            performance,
        })
    }

    /// Returns the options objects used to create this backend.
    pub fn options(&self) -> &WebGl2BackendOptions {
        &self.options
    }

    /// Returns the [`CursorShape`].
    pub fn cursor_shape(&self) -> &CursorShape {
        &self.options.cursor_shape
    }

    /// Set the [`CursorShape`].
    pub fn set_cursor_shape(mut self, shape: CursorShape) -> Self {
        self.options.cursor_shape = shape;
        self
    }

    /// Sets the canvas viewport and projection, reconfigures the terminal grid.
    pub fn resize_canvas(&mut self) -> Result<(), Error> {
        let size_px = self.beamterm.canvas_size();

        // resize the terminal grid and viewport
        self.beamterm.resize(size_px.0, size_px.1)?;

        // Update mouse handler dimensions if it exists
        if let Some(mouse_handler) = &self.hyperlink_mouse_handler {
            let (cols, rows) = self.beamterm.terminal_size();
            mouse_handler.update_dimensions(cols, rows);
        }

        // clear any hyperlink cells; we'll get them in the next draw call
        if let Some(hyperlink_cells) = &mut self.hyperlink_cells {
            let cell_count = self.beamterm.cell_count();
            hyperlink_cells.clear();
            hyperlink_cells.resize(cell_count, false);
        }

        Ok(())
    }

    /// Checks if the canvas size matches the display size and resizes it if necessary.
    fn check_canvas_resize(&mut self) -> Result<(), Error> {
        let canvas = self.beamterm.canvas();
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

    /// Updates the terminal grid with new cell content.
    fn update_grid<'a, I>(&mut self, content: I) -> Result<(), Error> where I: Iterator<Item = (u16, u16, &'a Cell)> {
        // If enabled, measures the time taken to synchronize the terminal buffer.
        self.measure_begin(SYNC_TERMINAL_BUFFER_MARK);

        // If hyperlink support is enabled, we need to track which cells are hyperlinks,
        // before passing the content to the beamterm renderer.
        if let Some(hyperlink_cells) = self.hyperlink_cells.as_mut() {
            let w = self.beamterm.terminal_size().0 as usize;

            // Mark any cells that have the hyperlink modifier set (don't blink!).
            // At this stage, we don't care about the actual cell content,
            // as we can extract it on demand.
            let cells = content.inspect(|(x, y, c)| {
                let idx = *y as usize * w + *x as usize;
                hyperlink_cells.set(idx, c.modifier.contains(HYPERLINK_MODIFIER));
            });
            let cells = cells.map(|(x, y, cell)| (x, y, cell_data(cell)));

            self.beamterm.update_cells_by_position(cells)
        } else {
            let cells = content.map(|(x, y, cell)| (x, y, cell_data(cell)));
            self.beamterm.update_cells_by_position(cells)
        }.map_err(Error::from)?;

        self.measure_end(SYNC_TERMINAL_BUFFER_MARK);

        Ok(())
    }

    /// Toggles the cursor visibility based on its current position.
    /// If there is no cursor position, it does nothing.
    fn toggle_cursor(&mut self) {
        if let Some(pos) = self.cursor_position {
            self.draw_cursor(pos);
        }
    }

    /// Draws the cursor at the specified position.
    fn draw_cursor(&mut self, pos: Position) {
        if let Some(c) = self.beamterm.grid().borrow_mut().cell_data_mut(pos.x, pos.y) {
            match self.options.cursor_shape {
                CursorShape::SteadyBlock => {
                    c.flip_colors();
                }
                CursorShape::SteadyUnderScore => {
                    // if the overall style is underlined, remove it, otherwise add it
                    c.style(c.get_style() ^ (GlyphEffect::Underline as u16));
                }
            }
        }
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

    /// Creates a mouse handler specifically for hyperlink clicks.
    fn create_hyperlink_mouse_handler(
        beamterm: &Beamterm,
        hyperlink_cells: Option<&BitVec>,
        callback: Rc<RefCell<dyn FnMut(&str)>>,
    ) -> Result<TerminalMouseHandler, Error> {
        let grid = beamterm.grid();
        let canvas = beamterm.canvas();
        let hyperlink_cells_clone = hyperlink_cells.map(|cells| cells.clone());
        
        let mouse_handler = TerminalMouseHandler::new(
            &canvas,
            grid,
            move |event: TerminalMouseEvent, grid: &beamterm_renderer::TerminalGrid| {
                // Only handle left mouse button clicks
                if event.event_type == MouseEventType::MouseUp && event.button == 0 {
                    if let Some(url) = Self::extract_hyperlink_url_static(
                        &hyperlink_cells_clone,
                        grid,
                        event.col,
                        event.row,
                    ) {
                        // Call the user's hyperlink callback
                        if let Ok(mut cb) = callback.try_borrow_mut() {
                            cb(&url);
                        }
                    }
                }
            },
        )?;

        Ok(mouse_handler)
    }

    /// Extracts hyperlink URL from grid coordinates (static version for use in closures).
    fn extract_hyperlink_url_static(
        hyperlink_cells: &Option<BitVec>,
        grid: &beamterm_renderer::TerminalGrid,
        start_col: u16,
        row: u16,
    ) -> Option<String> {
        let hyperlink_cells = hyperlink_cells.as_ref()?;
        let (cols, _) = grid.terminal_size();
        
        // Check if clicked cell is a hyperlink
        let start_idx = row as usize * cols as usize + start_col as usize;
        if !hyperlink_cells.get(start_idx).map(|b| *b).unwrap_or(false) {
            return None;
        }
        
        // Find hyperlink boundaries
        let (link_start, link_end) = Self::find_hyperlink_bounds_static(
            hyperlink_cells, start_col, row, cols
        )?;
        
        // Extract text using beamterm's grid
        Self::extract_text_from_grid_static(grid, link_start, link_end, row)
    }

    /// Finds the start and end boundaries of a hyperlink (static version).
    fn find_hyperlink_bounds_static(
        hyperlink_cells: &BitVec,
        start_col: u16,
        row: u16,
        cols: u16,
    ) -> Option<(u16, u16)> {
        let row_start_idx = row as usize * cols as usize;
        
        // Find start of hyperlink (scan left)
        let mut link_start = start_col;
        while link_start > 0 {
            let idx = row_start_idx + (link_start - 1) as usize;
            if !hyperlink_cells.get(idx).map(|b| *b).unwrap_or(false) {
                break;
            }
            link_start -= 1;
        }
        
        // Find end of hyperlink (scan right)
        let mut link_end = start_col;
        while link_end < cols - 1 {
            let idx = row_start_idx + (link_end + 1) as usize;
            if !hyperlink_cells.get(idx).map(|b| *b).unwrap_or(false) {
                break;
            }
            link_end += 1;
        }
        
        Some((link_start, link_end))
    }

    /// Extracts text from beamterm grid using get_text(CellQuery).
    fn extract_text_from_grid_static(
        grid: &beamterm_renderer::TerminalGrid,
        start_col: u16,
        end_col: u16,
        row: u16,
    ) -> Option<String> {
        // Create a selection query for the hyperlink range
        let query = select(SelectionMode::Block)
            .start((start_col, row))
            .end((end_col, row))
            .trim_trailing_whitespace(true);
            
        let text = grid.get_text(query);
        let trimmed = text.trim();
        
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }
}

impl Backend for WebGl2Backend {
    // Populates the buffer with the *updated* cell content.
    fn draw<'a, I>(&mut self, content: I) -> IoResult<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        if content.size_hint().1 != Some(0) {
            self.update_grid(content)?;
        }

        Ok(())
    }

    /// Flush the content to the screen.
    ///
    /// This function is called after the [`WebGl2Backend::draw`] function to
    /// actually render the content to the screen.
    fn flush(&mut self) -> IoResult<()> {
        self.check_canvas_resize()?;

        self.measure_begin(WEBGL_RENDER_MARK);
        
        // Flushes GPU buffers and render existing content to the canvas
        self.toggle_cursor(); // show cursor before rendering
        self.beamterm.render_frame().map_err(Error::from)?;
        self.toggle_cursor(); // restore cell to previous state
        
        self.measure_end(WEBGL_RENDER_MARK);
        
        Ok(())
    }

    fn hide_cursor(&mut self) -> IoResult<()> {
        self.cursor_position = None;
        Ok(())
    }

    fn show_cursor(&mut self) -> IoResult<()> {
        Ok(())
    }

    fn clear(&mut self) -> IoResult<()> {
        let cells = [CellData::new_with_style_bits(" ", 0, 0xffffff, 0x000000)]
            .into_iter()
            .cycle()
            .take(self.beamterm.cell_count());

        self.beamterm.update_cells(cells).map_err(Error::from)?;

        if let Some(hyperlink_cells) = &mut self.hyperlink_cells {
            hyperlink_cells.clear();
        }

        Ok(())
    }

    fn size(&self) -> IoResult<Size> {
        let (w, h) = self.beamterm.terminal_size();
        Ok(Size::new(w, h))
    }

    fn window_size(&mut self) -> IoResult<WindowSize> {
        let (cols, rows) = self.beamterm.terminal_size();
        let (w, h) = self.beamterm.canvas_size();

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
        self.cursor_position = Some(position.into());
        Ok(())
    }
}

/// Resolves foreground and background colors for a [`Cell`].
fn resolve_fg_bg_colors(cell: &Cell) -> (u32, u32) {
    let mut fg = cell.fg;
    let mut bg = cell.bg;

    if cell.modifier.contains(Modifier::REVERSED) {
        swap(&mut fg, &mut bg);
    }

    let mut c = cell.clone();
    c.set_fg(fg);
    c.set_bg(bg);

    (to_rgb(c.fg, 0xffffff), to_rgb(c.bg, 0x000000))
}

/// Converts a [`Cell`] into a [`CellData`] for the beamterm renderer.
fn cell_data(cell: &Cell) -> CellData {
    let (fg, bg) = resolve_fg_bg_colors(cell);
    CellData::new_with_style_bits(cell.symbol(), into_glyph_bits(cell.modifier), fg, bg)
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

impl std::fmt::Debug for WebGl2Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebGl2Backend")
            .field("beamterm", &"<beamterm>")
            .field("options", &self.options)
            .field("cursor_position", &self.cursor_position)
            .field("performance", &self.performance.is_some())
            .field("hyperlink_cells", &self.hyperlink_cells.as_ref().map(|c| c.len()))
            .field("hyperlink_mouse_handler", &self.hyperlink_mouse_handler.is_some())
            .field("hyperlink_callback", &self.hyperlink_callback.is_some())
            .finish()
    }
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
