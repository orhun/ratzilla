use bitvec::{bitvec, prelude::BitVec};
use ratatui::layout::Rect;
use sledgehammer_bindgen::bindgen;
use std::{
    io::Result as IoResult,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{
    backend::{
        color::{actual_bg_color, actual_fg_color},
        utils::*,
    },
    error::Error,
    render::WebBackend,
    CursorShape,
};
use ratatui::{
    backend::WindowSize,
    buffer::Cell,
    layout::{Position, Size},
    prelude::Backend,
    style::{Color, Modifier},
};
use web_sys::{
    js_sys::{Float64Array, Uint16Array},
    wasm_bindgen::{
        self,
        prelude::{wasm_bindgen, Closure},
        JsCast,
    },
    Element,
};

/// Options for the [`CanvasBackend`].
#[derive(Debug, Default)]
pub struct CanvasBackendOptions {
    /// The element ID.
    grid_id: Option<String>,
    /// Always clip foreground drawing to the cell rectangle. Helpful when
    /// dealing with out-of-bounds rendering from problematic fonts. Enabling
    /// this option may cause some performance issues when dealing with large
    /// numbers of simultaneous changes.
    always_clip_cells: bool,
    /// An optional string which sets a custom font for the canvas
    font_str: Option<String>,
}

impl CanvasBackendOptions {
    /// Constructs a new [`CanvasBackendOptions`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the element id of the canvas' parent element.
    pub fn grid_id(mut self, id: &str) -> Self {
        self.grid_id = Some(id.to_string());
        self
    }

    /// Sets the font that the canvas will use
    pub fn font(mut self, font: String) -> Self {
        self.font_str = Some(font);
        self
    }
}

// Mirrors usage in https://github.com/DioxusLabs/dioxus/blob/main/packages/interpreter/src/unified_bindings.rs
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    /// External JS class for managing the actual HTML canvas, context,
    /// and parent element.
    pub type RatzillaCanvas;

    #[wasm_bindgen(method)]
    /// Returns the cell width, cell height, and cell ascent in that order
    fn measure_text(this: &RatzillaCanvas, text: &str) -> Float64Array;

    #[wasm_bindgen(method)]
    fn get_canvas(this: &RatzillaCanvas) -> web_sys::HtmlCanvasElement;

    #[wasm_bindgen(method)]
    /// Returns the new number of cells in width and height in that order
    fn reinit_canvas(this: &RatzillaCanvas) -> Uint16Array;
}

impl Buffer {
    /// Converts the buffer to its baseclass
    pub fn ratzilla_canvas(&self) -> &RatzillaCanvas {
        use wasm_bindgen::prelude::JsCast;
        self.js_channel().unchecked_ref()
    }
}

#[bindgen]
mod js {
    #[extends(RatzillaCanvas)]
    /// Responsible for buffering the calls to the canvas and
    /// canvas context
    struct Buffer;

    const BASE: &str = r#"src/backend/canvas_import.js"#;

    fn save() {
        r#"
            this.ctx.save();
        "#
    }

    fn restore() {
        r#"
            this.ctx.restore();
        "#
    }

    fn begin_path() {
        r#"
            this.ctx.beginPath();
        "#
    }

    fn clip() {
        r#"
            this.ctx.clip();
        "#
    }

    fn rect(x: u16, y: u16, w: u16, h: u16) {
        r#"
            this.ctx.rect($x$, $y$, $w$, $h$)
        "#
    }

    fn fill_rect(x: u16, y: u16, w: u16, h: u16) {
        r#"
            this.ctx.fillRect($x$, $y$, $w$, $h$)
        "#
    }

    fn stroke_rect(x: u16, y: u16, w: u16, h: u16) {
        r#"
            this.ctx.strokeRect($x$, $y$, $w$, $h$)
        "#
    }

    fn fill_text(text: &str, x: u16, y: u16) {
        r#"
            this.ctx.fillText($text$, $x$, $y$);
        "#
    }

    fn set_fill_style_str(style: &str) {
        r#"
            this.ctx.fillStyle = $style$;
        "#
    }

    fn set_stroke_style_str(style: &str) {
        r#"
            this.ctx.strokeStyle = $style$;
        "#
    }

    fn create_canvas_in_element(parent: &str) {
        r#"
            this.parent = document.getElementById($parent$);
            if (this.parent == null) {
                this.parent = document.body;
            }
            this.canvas = document.createElement("canvas");
            this.canvas.width = this.parent.clientWidth;
            this.canvas.height = this.parent.clientHeight;
            this.parent.appendChild(this.canvas);
        "#
    }

    fn init_ctx(font_str: &str<u8, fontstr>) {
        r#"
            this.font_str = $font_str$;
            super.init_ctx();
        "#
    }
}

/// Canvas renderer.
struct Canvas {
    /// The buffer of draw calls to the canvas
    buffer: Buffer,
    /// Whether the canvas has been initialized.
    initialized: Rc<AtomicBool>,
    /// The inner HTML canvas element
    ///
    /// Use **only** for implementing `WebBackend`
    inner: web_sys::HtmlCanvasElement,
    /// Background color.
    background_color: Color,
    /// An optional string which sets a custom font for the canvas
    font_str: Option<String>,
    /// Width of a single cell.
    ///
    /// This will be used for multiplying the cell's x position to get the actual pixel
    /// position on the canvas.
    cell_width: f64,
    /// Height of a single cell.
    ///
    /// This will be used for multiplying the cell's y position to get the actual pixel
    /// position on the canvas.
    cell_height: f64,
    /// The font ascent of the `|` character as measured by the canvas
    cell_ascent: f64,
}

impl Canvas {
    /// Constructs a new [`Canvas`].
    fn new(
        parent_element: &str,
        background_color: Color,
        font_str: Option<String>,
    ) -> Result<Self, Error> {
        let initialized: Rc<AtomicBool> = Rc::new(false.into());
        let closure = Closure::<dyn FnMut(_)>::new({
            let initialized = Rc::clone(&initialized);
            move |_: web_sys::Event| {
                initialized.store(false, Ordering::Relaxed);
            }
        });
        web_sys::window()
            .unwrap()
            .set_onresize(Some(closure.as_ref().unchecked_ref()));
        closure.forget();

        let mut buffer = Buffer::default();
        buffer.create_canvas_in_element(parent_element);

        let mut canvas = Self {
            inner: buffer.ratzilla_canvas().get_canvas(),
            buffer,
            initialized,
            background_color,
            font_str,
            cell_width: 0.0,
            cell_height: 0.0,
            cell_ascent: 0.0,
        };

        canvas.init_ctx();
        canvas.buffer.flush();

        let font_measurement = canvas.buffer.ratzilla_canvas().measure_text("█");
        canvas.cell_width = font_measurement.get_index(0);
        canvas.cell_height = font_measurement.get_index(1);
        canvas.cell_ascent = font_measurement.get_index(2);

        Ok(canvas)
    }

    fn font_metrics(&self) -> Size {
        Size {
            width: self.cell_width as u16,
            height: self.cell_height as u16,
        }
    }

    fn init_ctx(&mut self) {
        self.buffer
            .init_ctx(self.font_str.as_deref().unwrap_or("16px monospace"));
    }
}

/// Canvas backend.
///
/// This backend renders the buffer onto a HTML canvas element.
pub struct CanvasBackend {
    /// Always clip foreground drawing to the cell rectangle. Helpful when
    /// dealing with out-of-bounds rendering from problematic fonts. Enabling
    /// this option may cause some performance issues when dealing with large
    /// numbers of simultaneous changes.
    always_clip_cells: bool,
    /// Current buffer.
    buffer: Vec<Vec<Cell>>,
    /// Changed buffer cells
    changed_cells: BitVec,
    /// Canvas.
    canvas: Canvas,
    /// Cursor position.
    cursor_position: Option<Position>,
    /// The cursor shape.
    cursor_shape: CursorShape,
    /// Draw cell boundaries with specified color.
    debug_mode: Option<String>,
}

impl CanvasBackend {
    /// Constructs a new [`CanvasBackend`].
    pub fn new() -> Result<Self, Error> {
        Self::new_with_options(CanvasBackendOptions::default())
    }

    /// Constructs a new [`CanvasBackend`] with the given options.
    pub fn new_with_options(mut options: CanvasBackendOptions) -> Result<Self, Error> {
        // Parent element of canvas (uses <body> unless specified)
        let parent = options.grid_id.as_deref().unwrap_or_default();

        let canvas = Canvas::new(parent, Color::Black, options.font_str.take())?;
        let buffer = get_sized_buffer_from_canvas(
            &canvas.buffer.ratzilla_canvas().get_canvas(),
            canvas.font_metrics(),
        );
        let changed_cells = bitvec![1; buffer.len() * buffer[0].len()];
        Ok(Self {
            always_clip_cells: options.always_clip_cells,
            buffer,
            changed_cells,
            canvas,
            cursor_position: None,
            cursor_shape: CursorShape::SteadyBlock,
            debug_mode: None,
        })
    }

    fn buffer_size(&self) -> Size {
        Size::new(self.buffer[0].len() as u16, self.buffer.len() as u16)
    }

    fn initialize(&mut self) -> Result<(), Error> {
        // TODO: Find a way to not use a Javascript array
        let new_buffer_size = self.canvas.buffer.ratzilla_canvas().reinit_canvas();

        let new_buffer_size = Size {
            width: new_buffer_size.get_index(0),
            height: new_buffer_size.get_index(1),
        };
        if self.buffer_size() != new_buffer_size {
            for line in &mut self.buffer {
                line.resize_with(new_buffer_size.width as usize, || Cell::default());
            }
            self.buffer
                .resize_with(new_buffer_size.height as usize, || {
                    vec![Cell::default(); new_buffer_size.width as usize]
                });
            self.changed_cells = bitvec![usize::MAX; self.buffer.len() * self.buffer[0].len()];
        }

        Ok(())
    }

    /// Sets the background color of the canvas.
    pub fn set_background_color(&mut self, color: Color) {
        self.canvas.background_color = color;
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

    /// Enable or disable debug mode to draw cells with a specified color.
    ///
    /// The format of the color is the same as the CSS color format, e.g.:
    /// - `#666`
    /// - `#ff0000`
    /// - `red`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ratzilla::CanvasBackend;
    /// let mut backend = CanvasBackend::new().unwrap();
    ///
    /// backend.set_debug_mode(Some("#666"));
    /// backend.set_debug_mode(Some("red"));
    /// ```
    pub fn set_debug_mode<T: Into<String>>(&mut self, color: Option<T>) {
        self.debug_mode = color.map(Into::into);
    }

    // Compare the current buffer to the previous buffer and updates the canvas
    // accordingly.
    //
    // If `force_redraw` is `true`, the entire canvas will be cleared and redrawn.
    //
    // NOTE: The draw_* functions each traverse the buffer once, instead of
    // traversing it once per cell; this is done to reduce the number of
    // WASM calls per cell.
    fn update_grid(&mut self, force_redraw: bool) -> Result<(), Error> {
        if force_redraw {
            self.initialize()?;
        }

        self.draw_background()?;
        self.draw_symbols()?;
        self.draw_cursor()?;
        if self.debug_mode.is_some() {
            self.draw_debug()?;
        }

        self.canvas.buffer.flush();
        self.changed_cells.set_elements(0x00);
        Ok(())
    }

    /// Draws the text symbols on the canvas.
    ///
    /// This method renders the textual content of each cell in the buffer, optimizing canvas operations
    /// by minimizing state changes across the WebAssembly boundary.
    ///
    /// # Optimization Strategy
    ///
    /// Rather than saving/restoring the canvas context for every cell (which would be expensive),
    /// this implementation:
    ///
    /// 1. Only processes cells that have changed since the last render.
    /// 2. Tracks the last foreground color used to avoid unnecessary style changes
    /// 3. Only creates clipping paths for potentially problematic glyphs (non-ASCII)
    /// or when `always_clip_cells` is enabled.
    fn draw_symbols(&mut self) -> Result<(), Error> {
        let changed_cells = &self.changed_cells;
        let mut index = 0;

        self.canvas.buffer.save();
        let mut last_color = None;
        for (y, line) in self.buffer.iter().enumerate() {
            for (x, cell) in line.iter().enumerate() {
                // Skip empty cells
                if !changed_cells.get(index).map(|c| *c).unwrap_or(true) || cell.symbol() == " " {
                    index += 1;
                    continue;
                }
                let color = actual_fg_color(cell);

                // We need to reset the canvas context state in two scenarios:
                // 1. When we need to create a clipping path (for potentially problematic glyphs)
                // 2. When the text color changes
                if self.always_clip_cells || !cell.symbol().is_ascii() {
                    self.canvas.buffer.restore();
                    self.canvas.buffer.save();

                    self.canvas.buffer.begin_path();
                    self.canvas.buffer.rect(
                        (x as f64 * self.canvas.cell_width) as _,
                        (y as f64 * self.canvas.cell_height) as _,
                        self.canvas.cell_width as _,
                        self.canvas.cell_height as _,
                    );
                    self.canvas.buffer.clip();

                    last_color = None; // reset last color to avoid clipping
                    let color = get_canvas_color(color, Color::White);
                    self.canvas.buffer.set_fill_style_str(&color);
                } else if last_color != Some(color) {
                    self.canvas.buffer.restore();
                    self.canvas.buffer.save();

                    last_color = Some(color);

                    let color = get_canvas_color(color, Color::White);
                    self.canvas.buffer.set_fill_style_str(&color);
                }

                self.canvas.buffer.fill_text(
                    cell.symbol(),
                    (x as f64 * self.canvas.cell_width) as _,
                    (y as f64 * self.canvas.cell_height + self.canvas.cell_ascent) as _,
                );

                index += 1;
            }
        }
        self.canvas.buffer.restore();

        Ok(())
    }

    /// Draws the background of the cells.
    ///
    /// This function uses [`RowColorOptimizer`] to optimize the drawing of the background
    /// colors by batching adjacent cells with the same color into a single rectangle.
    ///
    /// In other words, it accumulates "what to draw" until it finds a different
    /// color, and then it draws the accumulated rectangle.
    fn draw_background(&mut self) -> Result<(), Error> {
        let changed_cells = &self.changed_cells;
        self.canvas.buffer.save();

        let draw_region = |(rect, color, canvas): (Rect, Color, &mut Canvas)| {
            let color = get_canvas_color(color, canvas.background_color);

            canvas.buffer.set_fill_style_str(&color);
            canvas.buffer.fill_rect(
                (rect.x as f64 * canvas.cell_width) as _,
                (rect.y as f64 * canvas.cell_height) as _,
                (rect.width as f64 * canvas.cell_width) as _,
                (rect.height as f64 * canvas.cell_height) as _,
            );
        };

        let mut index = 0;
        for (y, line) in self.buffer.iter().enumerate() {
            let mut row_renderer = RowColorOptimizer::new();
            for (x, cell) in line.iter().enumerate() {
                if changed_cells.get(index).map(|c| *c).unwrap_or(true) {
                    // Only calls `draw_region` if the color is different from the previous one
                    row_renderer
                        .process_color((x, y), actual_bg_color(cell))
                        .map(|(rect, color)| draw_region((rect, color, &mut self.canvas)));
                } else {
                    // Cell is unchanged so we must flush any held region
                    // to avoid clearing the foreground (symbol) of the cell
                    row_renderer
                        .flush()
                        .map(|(rect, color)| draw_region((rect, color, &mut self.canvas)));
                }
                index += 1;
            }
            // Flush the remaining region after traversing the row
            row_renderer
                .flush()
                .map(|(rect, color)| draw_region((rect, color, &mut self.canvas)));
        }

        self.canvas.buffer.restore();

        Ok(())
    }

    /// Draws the cursor on the canvas.
    fn draw_cursor(&mut self) -> Result<(), Error> {
        if let Some(pos) = self.cursor_position {
            let cell = &self.buffer[pos.y as usize][pos.x as usize];

            if cell.modifier.contains(Modifier::UNDERLINED) {
                self.canvas.buffer.save();

                self.canvas.buffer.fill_text(
                    "_",
                    (pos.x as f64 * self.canvas.cell_width) as _,
                    (pos.y as f64 * self.canvas.cell_height + self.canvas.cell_ascent) as _,
                );

                self.canvas.buffer.restore();
            }
        }

        Ok(())
    }

    /// Draws cell boundaries for debugging.
    fn draw_debug(&mut self) -> Result<(), Error> {
        self.canvas.buffer.save();

        let color = self.debug_mode.as_ref().unwrap();
        for (y, line) in self.buffer.iter().enumerate() {
            for (x, _) in line.iter().enumerate() {
                self.canvas.buffer.set_stroke_style_str(color);
                self.canvas.buffer.stroke_rect(
                    (x as f64 * self.canvas.cell_width) as _,
                    (y as f64 * self.canvas.cell_height) as _,
                    self.canvas.cell_width as _,
                    self.canvas.cell_height as _,
                );
            }
        }

        self.canvas.buffer.restore();

        Ok(())
    }
}

impl Backend for CanvasBackend {
    // Populates the buffer with the given content.
    fn draw<'a, I>(&mut self, content: I) -> IoResult<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        for (x, y, cell) in content {
            let y = y as usize;
            let x = x as usize;
            if let Some(line) = self.buffer.get_mut(y) {
                line.get_mut(x).map(|c| *c = cell.clone());
                if let Some(mut cell) = self.changed_cells.get_mut((y * line.len()) + x) {
                    cell.set(true);
                }
            }
        }

        // Draw the cursor if set
        if let Some(pos) = self.cursor_position {
            let y = pos.y as usize;
            let x = pos.x as usize;
            if let Some(line) = self.buffer.get_mut(y) {
                if x < line.len() {
                    let cursor_style = self.cursor_shape.show(line[x].style());
                    line.get_mut(x).map(|c| c.set_style(cursor_style));
                    if let Some(mut cell) = self.changed_cells.get_mut((y * line.len()) + x) {
                        cell.set(true);
                    }
                }
            }
        }

        Ok(())
    }

    /// Flush the content to the screen.
    ///
    /// This function is called after the [`CanvasBackend::draw`] function to
    /// actually render the content to the screen.
    fn flush(&mut self) -> IoResult<()> {
        let initialized = self.canvas.initialized.swap(true, Ordering::Relaxed);
        if self.changed_cells.any() || !initialized {
            self.update_grid(!initialized)?;
        }

        Ok(())
    }

    fn hide_cursor(&mut self) -> IoResult<()> {
        if let Some(pos) = self.cursor_position {
            let y = pos.y as usize;
            let x = pos.x as usize;
            let line = &mut self.buffer[y];
            if x < line.len() {
                let style = self.cursor_shape.hide(line[x].style());
                line[x].set_style(style);
            }
        }
        self.cursor_position = None;
        Ok(())
    }

    fn show_cursor(&mut self) -> IoResult<()> {
        Ok(())
    }

    fn get_cursor(&mut self) -> IoResult<(u16, u16)> {
        Ok((0, 0))
    }

    fn set_cursor(&mut self, _x: u16, _y: u16) -> IoResult<()> {
        Ok(())
    }

    fn clear(&mut self) -> IoResult<()> {
        self.buffer
            .iter_mut()
            .flatten()
            .for_each(|c| *c = Cell::default());
        Ok(())
    }

    fn size(&self) -> IoResult<Size> {
        Ok(self.buffer_size())
    }

    fn window_size(&mut self) -> IoResult<WindowSize> {
        unimplemented!()
    }

    fn get_cursor_position(&mut self) -> IoResult<Position> {
        match self.cursor_position {
            None => Ok((0, 0).into()),
            Some(position) => Ok(position),
        }
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> IoResult<()> {
        let new_pos = position.into();
        if let Some(old_pos) = self.cursor_position {
            let y = old_pos.y as usize;
            let x = old_pos.x as usize;
            let line = &mut self.buffer[y];
            if x < line.len() && old_pos != new_pos {
                let style = self.cursor_shape.hide(line[x].style());
                line[x].set_style(style);
            }
        }
        self.cursor_position = Some(new_pos);
        Ok(())
    }
}

impl WebBackend for CanvasBackend {
    fn listening_element(&self) -> &Element {
        &self.canvas.inner
    }
}

/// Optimizes canvas rendering by batching adjacent cells with the same color into a single rectangle.
///
/// This reduces the number of draw calls to the canvas API by coalescing adjacent cells
/// with identical colors into larger rectangles, which is particularly beneficial for
/// WASM where calls are quiteexpensive.
struct RowColorOptimizer {
    /// The currently accumulating region and its color
    pending_region: Option<(Rect, Color)>,
}

impl RowColorOptimizer {
    /// Creates a new empty optimizer with no pending region.
    fn new() -> Self {
        Self {
            pending_region: None,
        }
    }

    /// Processes a cell with the given position and color.
    fn process_color(&mut self, pos: (usize, usize), color: Color) -> Option<(Rect, Color)> {
        if let Some((active_rect, active_color)) = self.pending_region.as_mut() {
            if active_color == &color {
                // Same color: extend the rectangle
                active_rect.width += 1;
            } else {
                // Different color: flush the previous region and start a new one
                let region = *active_rect;
                let region_color = *active_color;
                *active_rect = Rect::new(pos.0 as _, pos.1 as _, 1, 1);
                *active_color = color;
                return Some((region, region_color));
            }
        } else {
            // First color: create a new rectangle
            let rect = Rect::new(pos.0 as _, pos.1 as _, 1, 1);
            self.pending_region = Some((rect, color));
        }

        None
    }

    /// Finalizes and returns the current pending region, if any.
    fn flush(&mut self) -> Option<(Rect, Color)> {
        self.pending_region.take()
    }
}
