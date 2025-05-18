use bitvec::{bitvec, prelude::BitVec};
use std::io::Result as IoResult;
use std::mem::swap;
use compact_str::format_compact;
use crate::{backend::utils::*, error::Error, CursorShape};
use ratatui::{
    backend::WindowSize,
    buffer::Cell,
    layout::{Position, Size},
    prelude::Backend,
    style::{Color, Modifier},
};
use web_sys::{console, js_sys::{Boolean, Map}, wasm_bindgen::{JsCast, JsValue}, window};

/// Width of a single cell.
///
/// This will be used for multiplying the cell's x position to get the actual pixel
/// position on the canvas.
const CELL_WIDTH: f64 = 10.0;

/// Height of a single cell.
///
/// This will be used for multiplying the cell's y position to get the actual pixel
/// position on the canvas.
const CELL_HEIGHT: f64 = 19.0;

/// Options for the [`CanvasBackend`].
#[derive(Debug, Default)]
pub struct WebGl2BackendOptions {
    /// The element ID.
    grid_id: Option<String>,
    /// Override the automatically detected size.
    size: Option<(u32, u32)>,
}

impl WebGl2BackendOptions {
    /// Constructs a new [`CanvasBackendOptions`].
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
}

/// WebGl2 renderer.
// #[derive(Debug)]
struct WebGl2 {
    renderer: webgl2::Renderer,
    terminal_grid: webgl2::TerminalGrid,
    /// WebGl2 element.
    // inner: web_sys::HtmlCanvasElement,
    /// Background color.
    background_color: Color,
}

impl WebGl2 {
    /// Constructs a new [`Canvas`].
    fn new(
        document: web_sys::Document,
        parent_element: web_sys::Element,
        width: u32,
        height: u32,
        background_color: Color,
    ) -> Result<Self, Error> {
        let element = document.create_element("canvas")?;
        let canvas = element
            .clone()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .expect("Unable to cast canvas element");
        canvas.set_width(width);
        canvas.set_height(height);
        let context_options = Map::new();
        context_options.set(&JsValue::from_str("alpha"), &Boolean::from(JsValue::TRUE));
        context_options.set(
            &JsValue::from_str("desynchronized"),
            &Boolean::from(JsValue::TRUE),
        );
        // context.set_font("16px monospace");
        // context.set_text_baseline("top");
        parent_element.append_child(&element)?;

        let renderer = webgl2::Renderer::create_with_canvas(canvas)
            .expect("Unable to create WebGL2 renderer");

        let font_atlas = webgl2::FontAtlas::load_default(renderer.gl()).expect("Unable to load font");
        let cell_size = font_atlas.cell_size();

        let terminal_grid = webgl2::TerminalGrid::new(
            renderer.gl(),
            font_atlas,
            renderer.canvas_size(),
        ).expect("Unable to create terminal grid");

        terminal_grid.upload_ubo_data(renderer.gl(), renderer.canvas_size(), cell_size);

        Ok(Self {
            // inner: canvas,
            terminal_grid,
            renderer,
            background_color,

        })
    }
}

/// WebGl2 backend.
///
/// This backend renders the buffer onto a HTML canvas element.
// #[derive(Debug)]
pub struct WebGl2Backend {
    /// Whether the canvas has been initialized.
    initialized: bool,
    /// Current buffer.
    buffer: Vec<Cell>,
    /// Previous buffer.
    // prev_buffer: Vec<Cell>,
    /// Changed buffer cells
    changed_cells: BitVec,
    /// WebGl2 context.
    context: WebGl2,
    /// Cursor position.
    cursor_position: Option<Position>,
    /// The cursor shape.
    cursor_shape: CursorShape,
    /// Draw cell boundaries with specified color.
    debug_mode: Option<String>,
}

impl WebGl2Backend {
    /// Constructs a new [`CanvasBackend`].
    pub fn new() -> Result<Self, Error> {
        let (width, height) = get_raw_window_size();
        Self::new_with_size(width.into(), height.into())
    }

    /// Constructs a new [`CanvasBackend`] with the given size.
    pub fn new_with_size(width: u32, height: u32) -> Result<Self, Error> {
        Self::new_with_options(WebGl2BackendOptions {
            size: Some((width, height)),
            ..Default::default()
        })
    }

    /// Constructs a new [`CanvasBackend`] with the given options.
    pub fn new_with_options(options: WebGl2BackendOptions) -> Result<Self, Error> {
        let window = window().ok_or(Error::UnableToRetrieveWindow)?;
        let document = window.document().ok_or(Error::UnableToRetrieveDocument)?;

        // Parent element of canvas (uses <body> unless specified)
        let parent = match options.grid_id.as_ref() {
            Some(id) => document
                .get_element_by_id(id)
                .ok_or(Error::UnableToRetrieveBody)?,
            None => document.body().ok_or(Error::UnableToRetrieveBody)?.into(),
        };

        let (width, height) = options
            .size
            .unwrap_or_else(|| (parent.client_width() as u32, parent.client_height() as u32));

        let context = WebGl2::new(document, parent, width, height, Color::Black)?;

        // setup font atlas
        // let atlas_config = FontAtlasConfig::default();
        // let font_atlas = FontAtlas::load_default()

        let buffer = get_sized_buffer_from_terminal_grid(&context.terminal_grid);
        let changed_cells = bitvec![0; buffer.len()];
        Ok(Self {
            // prev_buffer: buffer.clone(),
            buffer,
            initialized: false,
            changed_cells,
            context,
            cursor_position: None,
            cursor_shape: CursorShape::SteadyBlock,
            debug_mode: None,
        })
    }

    /// Sets the background color of the canvas.
    pub fn set_background_color(&mut self, color: Color) {
        // todo: propagte to renderer
        self.context.background_color = color;
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
    /// use ratzilla::WebGl2Backend;
    /// let mut backend = WebGl2Backend::new().unwrap();
    ///
    /// backend.set_debug_mode(Some("#666"));
    /// backend.set_debug_mode(Some("red"));
    /// ```
    pub fn set_debug_mode<T: Into<String>>(&mut self, color: Option<T>) {
        // todo: alternate grid shader
        self.debug_mode = color.map(Into::into);
    }

    // Compare the current buffer to the previous buffer and updates the canvas
    // accordingly.
    //
    // If `force_redraw` is `true`, the entire canvas will be cleared and redrawn.
    fn update_grid(&mut self, force_redraw: bool) -> Result<(), Error> {
        // if force_redraw {
        //     self.canvas.context.clear_rect(
        //         0.0,
        //         0.0,
        //         self.canvas.inner.client_width() as f64,
        //         self.canvas.inner.client_height() as f64,
        //     );
        // }
        // self.canvas.context.translate(5_f64, 5_f64)?;

        // NOTE: The draw_* functions each traverses the buffer once, instead of
        // traversing it once per cell; this is done to reduce the number of
        // WASM calls per cell.

        let gl = self.context.renderer.gl();
        let terminal = &mut self.context.terminal_grid;
        let cells = self.buffer.iter().map(cell_data);
        terminal.update_cells(gl, cells).expect("Unable to update cells");

        // self.resolve_changed_cells(force_redraw);
        // self.draw_background()?;
        // self.draw_symbols()?;
        // self.draw_cursor()?;
        // if self.debug_mode.is_some() {
        //     self.draw_debug()?;
        // }

        Ok(())
    }

    // /// Updates the representation of the changed cells.
    // ///
    // /// This function updates the `changed_cells` vector to indicate which cells
    // /// have changed.
    // fn resolve_changed_cells(&mut self, force_redraw: bool) {
    //     self.prev_buffer.iter()
    //         .zip(self.buffer.iter())
    //         .enumerate()
    //         .filter(|(_, (prev, cell))| force_redraw || prev != cell)
    //         .for_each(|(i, _)| self.changed_cells.set(i, true));
    // }

    // /// Draws the cursor on the canvas. todo: restore
    // fn draw_cursor(&mut self) -> Result<(), Error> {
    //     if let Some(pos) = self.cursor_position {
    //         let cell = &self.buffer[pos.y as usize][pos.x as usize];
    //
    //         if cell.modifier.contains(Modifier::UNDERLINED) {
    //             self.canvas.context.save();
    //
    //             self.canvas.context.fill_text(
    //                 "_",
    //                 pos.x as f64 * CELL_WIDTH,
    //                 pos.y as f64 * CELL_HEIGHT,
    //             )?;
    //
    //             self.canvas.context.restore();
    //         }
    //     }
    //
    //     Ok(())
    // }

    // /// Draws cell boundaries for debugging. // todo: restore
    // fn draw_debug(&mut self) -> Result<(), Error> {
    //     self.canvas.context.save();
    //
    //     let color = self.debug_mode.as_ref().unwrap();
    //     for (y, line) in self.buffer.iter().enumerate() {
    //         for (x, _) in line.iter().enumerate() {
    //             self.canvas.context.set_stroke_style_str(color);
    //             self.canvas.context.stroke_rect(
    //                 x as f64 * CELL_WIDTH,
    //                 y as f64 * CELL_HEIGHT,
    //                 CELL_WIDTH,
    //                 CELL_HEIGHT,
    //             );
    //         }
    //     }
    //
    //     self.canvas.context.restore();
    //
    //     Ok(())
    // }
}

fn cell_data(cell: &Cell) -> webgl2::CellData {
    let mut fg = to_rgba(cell.fg);
    let mut bg = to_rgba(cell.bg);
    if cell.modifier.contains(Modifier::REVERSED) {
        swap(&mut fg, &mut bg);
    }

    webgl2::CellData::new(cell.symbol(), fg, bg,)
}

impl Backend for WebGl2Backend {
    // Populates the buffer with the given content.
    fn draw<'a, I>(&mut self, content: I) -> IoResult<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        let w = self.context.terminal_grid.terminal_size().0 as usize;

        let terminal = &mut self.context.terminal_grid;
        self.context.renderer.render(terminal);

        for (x, y, cell) in content {
            let y = y as usize;
            let x = x as usize;

            self.buffer[y * w + x] = cell.clone();
        }
        
        //     line.extend(std::iter::repeat_with(Cell::default).take(x.saturating_sub(line.len())));
        //     line[x] = cell.clone();
        // }
        //
        // // Draw the cursor if set
        // if let Some(pos) = self.cursor_position {
        //     let y = pos.y as usize;
        //     let x = pos.x as usize;
        //     let line = &mut self.buffer[y];
        //     if x < line.len() {
        //         let cursor_style = self.cursor_shape.show(line[x].style());
        //         line[x].set_style(cursor_style);
        //     }
        // }

        Ok(())
    }

    /// Flush the content to the screen.
    ///
    /// This function is called after the [`CanvasBackend::draw`] function to
    /// actually render the content to the screen.
    fn flush(&mut self) -> IoResult<()> {
        // Only runs once.
        if !self.initialized {
            self.update_grid(true)?;
            // self.prev_buffer = self.buffer.clone();
            self.initialized = true;
            return Ok(());
        }

        // if self.buffer != self.prev_buffer {
            self.update_grid(false)?;
        // }

        // self.prev_buffer = self.buffer.clone();

        Ok(())
    }

    fn hide_cursor(&mut self) -> IoResult<()> {
        // if let Some(pos) = self.cursor_position {
        //     let y = pos.y as usize;
        //     let x = pos.x as usize;
        //     let line = &mut self.buffer[y];
        //     if x < line.len() {
        //         let style = self.cursor_shape.hide(line[x].style());
        //         line[x].set_style(style);
        //     }
        // }
        // self.cursor_position = None;
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
        // todo: clear canvas
        Ok(())
    }

    fn size(&self) -> IoResult<Size> {
        let (w, h) = self.context.terminal_grid.terminal_size();
        Ok(Size::new(w as _, h as _))
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
        // let new_pos = position.into();
        // if let Some(old_pos) = self.cursor_position {
        //     let y = old_pos.y as usize;
        //     let x = old_pos.x as usize;
        //     let line = &mut self.buffer[y];
        //     if x < line.len() && old_pos != new_pos {
        //         let style = self.cursor_shape.hide(line[x].style());
        //         line[x].set_style(style);
        //     }
        // }
        // self.cursor_position = Some(new_pos);
        Ok(())
    }
}



/// Returns a buffer based on the `TerminalGrid`.
fn get_sized_buffer_from_terminal_grid(grid: &webgl2::TerminalGrid) -> Vec<Cell> {
    vec![Cell::default(); grid.cell_count()]
}

fn to_rgba(color: Color) -> u32 {
    let c = match color {
        Color::Rgb(r, g, b) => ((r as u32) << 24) | ((g as u32) << 16) | (b as u32),
        Color::Reset => 0x00000000,
        Color::Black => 0x00000000,
        Color::Red => 0x80000000,
        Color::Green => 0x00800000,
        Color::Yellow => 0x80800000,
        Color::Blue => 0x00008000,
        Color::Magenta => 0x80008000,
        Color::Cyan => 0x00808000,
        Color::Gray => 0xc0c0c000,
        Color::DarkGray => 0x80808000,
        Color::LightRed => 0xff000000,
        Color::LightGreen => 0x00ff0000,
        Color::LightYellow => 0xffff0000,
        Color::LightBlue => 0x0000ff00,
        Color::LightMagenta => 0xff00ff00,
        Color::LightCyan => 0x00ffff00,
        Color::White => 0xffffff00,
        Color::Indexed(code) => {
            panic!("Indexed colors are not supported atm");
        }
    };

    c | 0xff // alpha to opaque
}