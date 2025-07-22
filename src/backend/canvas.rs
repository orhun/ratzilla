use ratatui::layout::Rect;
use sledgehammer_bindgen::bindgen;
use std::{
    io::Result as IoResult,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};
use unicode_width::UnicodeWidthStr;

use crate::{
    backend::color::{actual_bg_color, actual_fg_color, to_rgb},
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
    js_sys::Uint16Array,
    wasm_bindgen::{
        self,
        prelude::{wasm_bindgen, Closure},
        JsCast,
    },
    Element,
};

/// Options for the [`CanvasBackend`].
#[derive(Debug)]
pub struct CanvasBackendOptions {
    /// The element ID.
    grid_id: Option<String>,
    /// Always clip foreground drawing to the cell rectangle. Helpful when
    /// dealing with out-of-bounds rendering from problematic fonts. Enabling
    /// this option may cause some performance issues when dealing with large
    /// numbers of simultaneous changes.
    always_clip_cells: bool,
    /// Modifiers which may be used in rendering. Allows for the disabling
    /// of things like italics, which my not be available in some fonts
    /// like Fira Code
    enabled_modifiers: Modifier,
    /// An optional string which sets a custom font for the canvas
    font_str: Option<String>,
}

impl Default for CanvasBackendOptions {
    fn default() -> Self {
        Self {
            grid_id: None,
            always_clip_cells: false,
            enabled_modifiers: Modifier::BOLD
                | Modifier::ITALIC
                | Modifier::UNDERLINED
                | Modifier::REVERSED
                | Modifier::CROSSED_OUT,
            font_str: None,
        }
    }
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

    /// Enable modifiers for rendering, all modifiers that are supported
    /// are enabled by default
    pub fn enable_modifiers(mut self, modifiers: Modifier) -> Self {
        self.enabled_modifiers |= modifiers;
        self
    }

    /// Disable modifiers in rendering, allows for things like
    /// italics to be disabled if your chosen font doesn't support
    /// them
    pub fn disable_modifiers(mut self, modifiers: Modifier) -> Self {
        self.enabled_modifiers &= !modifiers;
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
    /// Does the initial construction of the RatzillaCanvas class
    ///
    /// `sledgehammer_bindgen` only lets you have an empty constructor,
    /// so we must initialize the class after construction
    fn create_canvas_in_element(
        this: &RatzillaCanvas,
        parent: &str,
        font_str: &str,
        background_color: u32,
    );

    #[wasm_bindgen(method)]
    /// Returns the cell width, cell height, and cell baseline in that order
    fn measure_text(this: &RatzillaCanvas) -> Uint16Array;

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

    const BASE: &str = r#"src/backend/ratzilla_canvas.js"#;

    fn clear_rect() {
        r#"
            this.ctx.fillStyle = this.backgroundColor;
            this.ctx.fillRect(
                0, 0, this.canvas.width, this.canvas.height
            );
        "#
    }

    fn save() {
        r#"
            this.ctx.save();
        "#
    }

    fn restore() {
        r#"
            this.bold = false;
            this.italic = false;
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

    fn font_modifiers(modifiers: u16) {
        r#"
            this.bold = (modifiers & 0b0000_0000_0001) != 0;
            this.italic = (modifiers & 0b0000_0000_0100) != 0;
            this.init_font();
        "#
    }

    fn rect(x: u16, y: u16, w: u16, h: u16) {
        r#"
            this.ctx.rect($x$, $y$, $w$, $h$);
        "#
    }

    fn fill() {
        r#"
            this.ctx.fill();
        "#
    }

    fn stroke() {
        r#"
            this.ctx.stroke();
        "#
    }

    fn fill_rect(x: u16, y: u16, w: u16, h: u16) {
        r#"
            this.ctx.fillRect($x$, $y$, $w$, $h$);
        "#
    }

    fn stroke_rect(x: u16, y: u16, w: u16, h: u16) {
        r#"
            this.ctx.strokeRect($x$, $y$, $w$, $h$);
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

    fn set_fill_style(style: u32) {
        r#"
            this.ctx.fillStyle = `#\${$style$.toString(16).padStart(6, '0')}`;
        "#
    }

    fn set_stroke_style_str(style: &str) {
        r#"
            this.ctx.strokeStyle = $style$;
        "#
    }

    fn set_stroke_style(style: u32) {
        r#"
            this.ctx.strokeStyle = `#\${$style$.toString(16).padStart(6, '0')}`;
        "#
    }
}

/// Canvas renderer.
struct Canvas {
    /// The buffer of draw calls to the canvas
    buffer: Buffer,
    /// Whether the canvas has been initialized.
    initialized: Rc<AtomicBool>,
    /// Modifiers which may be used in rendering. Allows for the disabling
    /// of things like italics, which my not be available in some fonts
    /// like Fira Code
    enabled_modifiers: Modifier,
    /// The inner HTML canvas element
    ///
    /// Use **only** for implementing `WebBackend`
    inner: web_sys::HtmlCanvasElement,
    /// Background color.
    background_color: Color,
    /// Width of a single cell.
    ///
    /// This will be used for multiplying the cell's x position to get the actual pixel
    /// position on the canvas.
    cell_width: u16,
    /// Height of a single cell.
    ///
    /// This will be used for multiplying the cell's y position to get the actual pixel
    /// position on the canvas.
    cell_height: u16,
    /// The font descent as measured by the canvas
    cell_baseline: u16,
    /// The font descent as measured by the canvas
    underline_pos: u16,
    /// Whether an actual change has been committed to the canvas
    /// aside from the background
    begun_drawing: bool,
    /// The fill style that the canvas should be set to, given drawing
    /// has begun
    fill_style: Color,
    /// The font modifiers that the canvas should be set to, given drawing
    /// has begun
    modifier: Modifier,
}

impl Canvas {
    /// Constructs a new [`Canvas`].
    fn new(
        parent_element: &str,
        background_color: Color,
        font_str: Option<String>,
        enabled_modifiers: Modifier,
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

        let buffer = Buffer::default();
        buffer.ratzilla_canvas().create_canvas_in_element(
            parent_element,
            font_str.as_deref().unwrap_or("16px monospace"),
            to_rgb(background_color, 0x000000),
        );

        let mut canvas = Self {
            inner: buffer.ratzilla_canvas().get_canvas(),
            buffer,
            initialized,
            enabled_modifiers,
            background_color,
            cell_width: 0,
            cell_height: 0,
            cell_baseline: 0,
            underline_pos: 0,
            begun_drawing: false,
            modifier: Modifier::empty(),
            fill_style: Color::default(),
        };

        let font_measurement = canvas.buffer.ratzilla_canvas().measure_text();
        canvas.cell_width = font_measurement.get_index(0);
        canvas.cell_height = font_measurement.get_index(1);
        canvas.cell_baseline = font_measurement.get_index(2);
        canvas.underline_pos = font_measurement.get_index(3);

        Ok(canvas)
    }

    fn draw_rect_modifiers(&mut self, region: Rect, modifiers: Modifier) {
        for modifier in modifiers {
            match modifier {
                Modifier::UNDERLINED => {
                    // self.buffer.stroke_rect(
                    //     region.x * self.cell_width,
                    //     region.y * self.cell_height,
                    //     region.width * self.cell_width,
                    //     self.cell_height,
                    // );
                    self.buffer.fill_rect(
                        region.x * self.cell_width,
                        region.y * self.cell_height + self.underline_pos,
                        region.width * self.cell_width,
                        1,
                    );
                }
                Modifier::CROSSED_OUT => {
                    // self.buffer.stroke_rect(
                    //     region.x * self.cell_width,
                    //     region.y * self.cell_height,
                    //     region.width * self.cell_width,
                    //     self.cell_height,
                    // );
                    self.buffer.fill_rect(
                        region.x * self.cell_width,
                        region.y * self.cell_height + self.cell_height / 2,
                        region.width * self.cell_width,
                        1,
                    );
                }
                _ => {}
            }
        }
    }

    fn begin_drawing(&mut self) {
        if !self.begun_drawing {
            self.buffer.save();
            self.buffer.clip();
            self.begun_drawing = true;
            let color = to_rgb(self.fill_style, 0xFFFFFF);
            self.buffer.set_fill_style(color);
            self.buffer.font_modifiers(self.modifier.bits());
        }
    }

    fn end_drawing(&mut self) {
        self.fill_style = Color::default();
        self.modifier = Modifier::empty();
        if self.begun_drawing {
            self.buffer.restore();
            self.begun_drawing = false;
        }
    }

    fn bold(&mut self) {
        self.modifier |= Modifier::BOLD;
        if self.begun_drawing {
            self.buffer.font_modifiers(self.modifier.bits());
        }
    }

    fn italic(&mut self) {
        self.modifier |= Modifier::ITALIC;
        if self.begun_drawing {
            self.buffer.font_modifiers(self.modifier.bits());
        }
    }

    fn bolditalic(&mut self) {
        self.modifier |= Modifier::ITALIC | Modifier::BOLD;
        if self.begun_drawing {
            self.buffer.font_modifiers(self.modifier.bits());
        }
    }

    fn unbold(&mut self) {
        self.modifier &= !Modifier::BOLD;
        if self.begun_drawing {
            self.buffer.font_modifiers(self.modifier.bits());
        }
    }

    fn unitalic(&mut self) {
        self.modifier &= !Modifier::ITALIC;
        if self.begun_drawing {
            self.buffer.font_modifiers(self.modifier.bits());
        }
    }

    fn unbolditalic(&mut self) {
        self.modifier &= !(Modifier::ITALIC | Modifier::BOLD);
        if self.begun_drawing {
            self.buffer.font_modifiers(self.modifier.bits());
        }
    }

    fn set_fill_style_lazy(&mut self, color: Color) {
        self.fill_style = color;
        if self.begun_drawing {
            let color = to_rgb(self.fill_style, 0xFFFFFF);
            self.buffer.set_fill_style(color);
        }
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
    /// The size of the current screen in cells
    buffer: Vec<Vec<Cell>>,
    /// Canvas.
    canvas: Canvas,
    /// Cursor position.
    cursor_position: Option<Position>,
    /// Whether the cursor has been drawn to the screen yet
    cursor_shown: bool,
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

        let canvas = Canvas::new(
            parent,
            Color::Black,
            options.font_str.take(),
            options.enabled_modifiers,
        )?;
        Ok(Self {
            always_clip_cells: options.always_clip_cells,
            canvas,
            buffer: Vec::new(),
            cursor_position: None,
            cursor_shown: false,
            cursor_shape: CursorShape::SteadyBlock,
            debug_mode: None,
        })
    }

    fn buffer_size(&self) -> Size {
        Size {
            width: self.buffer.get(0).map(|b| b.len()).unwrap_or(0) as u16,
            height: self.buffer.len() as u16,
        }
    }

    fn initialize(&mut self) -> Result<(), Error> {
        // TODO: Find a way to not use a Javascript array
        let new_buffer_size = self.canvas.buffer.ratzilla_canvas().reinit_canvas();

        let new_buffer_size = Size {
            width: new_buffer_size.get_index(0),
            height: new_buffer_size.get_index(1),
        };

        if self.buffer_size() != new_buffer_size {
            let new_buffer_width = new_buffer_size.width as usize;
            let new_buffer_height = new_buffer_size.height as usize;

            for line in &mut self.buffer {
                line.resize_with(new_buffer_width, || Cell::default());
            }
            self.buffer.resize_with(new_buffer_height, || {
                vec![Cell::default(); new_buffer_width]
            });
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

    /// Draws cell boundaries for debugging.
    fn draw_debug(&mut self) -> Result<(), Error> {
        self.canvas.buffer.save();

        let color = self.debug_mode.as_ref().unwrap();
        let buffer_size = self.buffer_size();
        for y in 0..buffer_size.height {
            for x in 0..buffer_size.width {
                self.canvas.buffer.set_stroke_style_str(color);
                self.canvas.buffer.stroke_rect(
                    x * self.canvas.cell_width,
                    y * self.canvas.cell_height,
                    self.canvas.cell_width,
                    self.canvas.cell_height,
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
        let initialized = self.canvas.initialized.swap(true, Ordering::Relaxed);

        if !initialized {
            self.initialize()?;
        }

        // self.canvas.buffer.clear_rect();

        let draw_region = |(rect, bg_color, canvas, cell_buffer): (
            Rect,
            Color,
            &mut Canvas,
            &mut Vec<(u16, u16, &Cell, Modifier)>,
        )| {
            let width: u16 = cell_buffer
                .iter()
                .map(|(_, _, c, _)| c.symbol().width() as u16)
                .sum();

            canvas.buffer.set_fill_style(to_rgb(bg_color, 0x000000));
            // canvas.buffer.set_stroke_style(0xFF0000);
            canvas.buffer.begin_path();
            canvas.buffer.rect(
                rect.x * canvas.cell_width,
                rect.y * canvas.cell_height,
                width * canvas.cell_width,
                rect.height * canvas.cell_height,
            );
            canvas.buffer.fill();
            // canvas.buffer.stroke();

            // Draws the text symbols on the canvas.
            //
            // This method renders the textual content of each cell in the buffer, optimizing canvas operations
            // by minimizing state changes across the WebAssembly boundary.
            //
            // # Optimization Strategy
            //
            // Rather than saving/restoring the canvas context for every cell (which would be expensive),
            // this implementation:
            //
            // 1. Only processes cells that have changed since the last render.
            // 2. Tracks the last foreground color used to avoid unnecessary style changes
            // 3. Only creates cell-level clipping paths when `always_clip_cells` is enabled
            let mut last_color = None;
            let mut last_modifier = Modifier::empty();
            let mut underline_optimizer: RowOptimizer<Modifier> = RowOptimizer::new();
            for (x, y, cell, modifiers) in cell_buffer.drain(..) {
                let fg_color =
                    actual_fg_color(cell, modifiers, Color::White, canvas.background_color);

                if self.always_clip_cells {
                    canvas.begin_drawing();
                    canvas.buffer.restore();
                    canvas.buffer.save();

                    let symbol_width = cell.symbol().width() as u16;
                    canvas.buffer.begin_path();
                    canvas.buffer.rect(
                        x * canvas.cell_width,
                        y * canvas.cell_height,
                        symbol_width * canvas.cell_width,
                        canvas.cell_height,
                    );
                    canvas.buffer.clip();

                    last_color = None;
                    last_modifier = Modifier::empty();
                }

                if last_color != Some(fg_color) {
                    last_color = Some(fg_color);

                    if let Some((region, modifiers)) = underline_optimizer.flush() {
                        if !modifiers.is_empty() {
                            canvas.begin_drawing();
                            canvas.draw_rect_modifiers(region, modifiers);
                        }
                    }
                    canvas.set_fill_style_lazy(fg_color);
                }

                if let Some((region, modifiers)) = underline_optimizer.process(
                    (x, y),
                    modifiers & (Modifier::UNDERLINED | Modifier::CROSSED_OUT),
                ) {
                    if !modifiers.is_empty() {
                        canvas.begin_drawing();
                        canvas.draw_rect_modifiers(region, modifiers);
                    }
                }

                let removed_modifiers = last_modifier - modifiers;

                match removed_modifiers & (Modifier::BOLD | Modifier::ITALIC) {
                    Modifier::BOLD => canvas.unbold(),
                    Modifier::ITALIC => canvas.unitalic(),
                    modifier if modifier.is_empty() => {}
                    _ => canvas.unbolditalic(),
                }

                let added_modifiers = modifiers - last_modifier;

                match added_modifiers & (Modifier::BOLD | Modifier::ITALIC) {
                    Modifier::BOLD => canvas.bold(),
                    Modifier::ITALIC => canvas.italic(),
                    modifier if modifier.is_empty() => {}
                    _ => canvas.bolditalic(),
                }

                last_modifier = modifiers;

                if fg_color != bg_color && cell.symbol() != " " {
                    // Very useful symbol positioning formulas from here
                    // https://github.com/ghostty-org/ghostty/blob/a88689ca754a6eb7dce6015b85ccb1416b5363d8/src/Surface.zig#L1589C5-L1589C10
                    canvas.begin_drawing();
                    canvas.buffer.fill_text(
                        cell.symbol(),
                        x * canvas.cell_width,
                        y * canvas.cell_height + canvas.cell_height - canvas.cell_baseline,
                    );
                }
            }

            if let Some((region, modifiers)) = underline_optimizer.flush() {
                if !modifiers.is_empty() {
                    canvas.begin_drawing();
                    canvas.draw_rect_modifiers(region, modifiers);
                }
            }
            canvas.end_drawing();
        };

        let mut bg_optimizer = RowOptimizer::new();
        let mut cell_buffer = Vec::new();
        for (x, y, cell) in content {
            let mut modifiers = cell.modifier;
            {
                let x = x as usize;
                let y = y as usize;
                if let Some(line) = self.buffer.get_mut(y) {
                    line.get_mut(x).map(|c| *c = cell.clone());
                }
            }

            if self
                .cursor_position
                .map(|pos| pos.x == x && pos.y == y)
                .unwrap_or_default()
            {
                let cursor_modifiers = self.cursor_shape.show(modifiers);
                modifiers = cursor_modifiers;
            }

            modifiers &= self.canvas.enabled_modifiers;

            // Draws the background of the cells.
            //
            // This function uses [`RowColorOptimizer`] to optimize the drawing of the background
            // colors by batching adjacent cells with the same color into a single rectangle.
            //
            // In other words, it accumulates "what to draw" until it finds a different
            // color, and then it draws the accumulated rectangle.
            //
            // Only calls `draw_region` if the color is different from the
            // previous one, or if we have advanced past the last y position,
            // or if we have advanced more than one x position
            bg_optimizer
                .process(
                    (x, y),
                    actual_bg_color(cell, modifiers, Color::White, self.canvas.background_color),
                )
                .map(|(rect, color)| {
                    draw_region((rect, color, &mut self.canvas, &mut cell_buffer))
                });

            cell_buffer.push((x, y, cell, modifiers));
        }

        // Flush the remaining region after traversing the changed cells
        bg_optimizer
            .flush()
            .map(|(rect, color)| draw_region((rect, color, &mut self.canvas, &mut cell_buffer)));

        Ok(())
    }

    /// Flush the content to the screen.
    ///
    /// This function is called after the [`CanvasBackend::draw`] function to
    /// actually render the content to the screen.
    fn flush(&mut self) -> IoResult<()> {
        if self.debug_mode.is_some() {
            self.draw_debug()?;
        }

        self.canvas.buffer.flush();

        Ok(())
    }

    fn hide_cursor(&mut self) -> IoResult<()> {
        // Redraw the cell under the cursor, but without
        // the cursor style
        if self.cursor_shown {
            if let Some(pos) = self.cursor_position.take() {
                let x = pos.x as usize;
                let y = pos.y as usize;
                if let Some(line) = self.buffer.get(y) {
                    if let Some(cell) = line.get(x).cloned() {
                        self.draw([(pos.x, pos.y, &cell)].into_iter())?;
                    }
                }
            }
            self.cursor_shown = false;
        }
        Ok(())
    }

    fn show_cursor(&mut self) -> IoResult<()> {
        // Redraw the new cell under the cursor, but with
        // the cursor style
        if !self.cursor_shown {
            if let Some(pos) = self.cursor_position {
                let x = pos.x as usize;
                let y = pos.y as usize;
                if let Some(line) = self.buffer.get(y) {
                    if let Some(cell) = line.get(x).cloned() {
                        self.draw([(pos.x, pos.y, &cell)].into_iter())?;
                    }
                }
            }
            self.cursor_shown = true;
        }
        Ok(())
    }

    fn get_cursor_position(&mut self) -> IoResult<Position> {
        match self.cursor_position {
            None => Ok((0, 0).into()),
            Some(position) => Ok(position),
        }
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> IoResult<()> {
        let position = position.into();
        if Some(position) != self.cursor_position {
            self.hide_cursor()?;
            self.cursor_position = Some(position.into());
            self.show_cursor()?;
        }
        Ok(())
    }

    fn clear(&mut self) -> IoResult<()> {
        self.canvas.buffer.clear_rect();
        self.buffer
            .iter_mut()
            .flatten()
            .for_each(|c| *c = Cell::default());
        Ok(())
    }

    fn size(&self) -> IoResult<Size> {
        if self.canvas.initialized.load(Ordering::Relaxed) {
            Ok(self.buffer_size())
        } else {
            let new_buffer_size = self.canvas.buffer.ratzilla_canvas().reinit_canvas();
            let new_buffer_size = Size {
                width: new_buffer_size.get_index(0),
                height: new_buffer_size.get_index(1),
            };
            Ok(new_buffer_size)
        }
    }

    fn window_size(&mut self) -> IoResult<WindowSize> {
        unimplemented!()
    }
}

impl WebBackend for CanvasBackend {
    fn listening_element(&self) -> &Element {
        &self.canvas.inner
    }
}

/// Optimizes canvas rendering by batching adjacent cells with the same data into a single rectangle.
///
/// This reduces the number of draw calls to the canvas API by coalescing adjacent cells
/// with identical data into larger rectangles, which is particularly beneficial for
/// WASM where calls are quiteexpensive.
struct RowOptimizer<T> {
    /// The currently accumulating region and its data
    pending_region: Option<(Rect, T)>,
}

impl<T> RowOptimizer<T> {
    /// Creates a new empty optimizer with no pending region.
    fn new() -> Self {
        Self {
            pending_region: None,
        }
    }

    /// Finalizes and returns the current pending region, if any.
    fn flush(&mut self) -> Option<(Rect, T)> {
        self.pending_region.take()
    }
}

impl<T: PartialEq + Eq + Copy> RowOptimizer<T> {
    /// Processes a cell with the given position and data.
    fn process(&mut self, pos: (u16, u16), data: T) -> Option<(Rect, T)> {
        if let Some((active_rect, active_data)) = self.pending_region.as_mut() {
            if active_data == &data && pos.0 == active_rect.right() && pos.1 == active_rect.y {
                // Same data: extend the rectangle
                active_rect.width += 1;
            } else {
                // Different data: flush the previous region and start a new one
                let region = *active_rect;
                let region_data = *active_data;
                *active_rect = Rect::new(pos.0, pos.1, 1, 1);
                *active_data = data;
                return Some((region, region_data));
            }
        } else {
            // First data: create a new rectangle
            let rect = Rect::new(pos.0, pos.1, 1, 1);
            self.pending_region = Some((rect, data));
        }

        None
    }
}
