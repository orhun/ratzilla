use std::io::Result as IoResult;

use ratatui::{
    backend::WindowSize,
    buffer::Cell,
    layout::{Position, Size},
    prelude::Backend,
    style::{Color, Modifier},
};
use web_sys::{
    js_sys::{Boolean, Map},
    wasm_bindgen::{JsCast, JsValue},
    window,
};

use crate::{backend::utils::*, error::Error, CursorShape};

/// Canvas renderer.
#[derive(Debug)]
struct Canvas {
    /// Canvas element.
    inner: web_sys::HtmlCanvasElement,
    /// Rendering context.
    context: web_sys::CanvasRenderingContext2d,
    /// Background color.
    background_color: Color,
}

impl Canvas {
    /// Constructs a new [`Canvas`].
    fn new(
        document: web_sys::Document,
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
        let context = canvas
            .get_context_with_context_options("2d", &context_options)?
            .ok_or_else(|| Error::UnableToRetrieveCanvasContext)?
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .expect("Unable to cast canvas context");

        context.set_font("16px monospace");
        context.set_text_baseline("top");
        let body = document.body().ok_or(Error::UnableToRetrieveBody)?;
        body.append_child(&element)?;
        Ok(Self {
            inner: canvas,
            context,
            background_color,
        })
    }
}

/// Canvas backend.
///
/// This backend renders the buffer onto a HTML canvas element.
#[derive(Debug)]
pub struct CanvasBackend {
    /// Whether the canvas has been initialized.
    initialized: bool,
    /// Current buffer.
    buffer: Vec<Vec<Cell>>,
    /// Previous buffer.
    prev_buffer: Vec<Vec<Cell>>,
    /// Canvas.
    canvas: Canvas,
    /// Cursor position.
    cursor_position: Option<Position>,
    /// The cursor shape.
    cursor_shape: CursorShape,
}

impl CanvasBackend {
    /// Constructs a new [`CanvasBackend`].
    pub fn new() -> Result<Self, Error> {
        let (width, height) = get_raw_window_size();
        Self::new_with_size(width.into(), height.into())
    }

    /// Constructs a new [`CanvasBackend`] with the given size.
    pub fn new_with_size(width: u32, height: u32) -> Result<Self, Error> {
        let window = window().ok_or(Error::UnableToRetrieveWindow)?;
        let document = window.document().ok_or(Error::UnableToRetrieveDocument)?;
        let canvas = Canvas::new(document, width, height, Color::Black)?;
        Ok(Self {
            buffer: get_sized_buffer_from_canvas(&canvas.inner),
            prev_buffer: get_sized_buffer_from_canvas(&canvas.inner),
            initialized: false,
            canvas,
            cursor_position: None,
            cursor_shape: CursorShape::SteadyBlock,
        })
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

    // Compare the current buffer to the previous buffer and updates the canvas
    // accordingly.
    //
    // If `force_redraw` is `true`, the entire canvas will be cleared and redrawn.
    fn update_grid(&mut self, force_redraw: bool) -> Result<(), Error> {
        if force_redraw {
            self.canvas.context.clear_rect(
                0.0,
                0.0,
                self.canvas.inner.client_width() as f64,
                self.canvas.inner.client_height() as f64,
            );
        }
        self.canvas.context.translate(5_f64, 5_f64)?;
        let xmul = 10.0;
        let ymul = 19.0;
        for (y, line) in self.buffer.iter().enumerate() {
            for (x, cell) in line.iter().enumerate() {
                if cell != &self.prev_buffer[y][x] || force_redraw {
                    let colors = get_cell_color_for_canvas(cell, self.canvas.background_color);
                    // Save the current state of the canvas context
                    self.canvas.context.save();

                    // Background
                    self.canvas.context.set_fill_style_str(colors.1.as_str());
                    self.canvas
                        .context
                        .fill_rect(x as f64 * xmul, y as f64 * ymul, xmul, ymul);

                    // Apply clipping for the text
                    self.canvas.context.begin_path();
                    self.canvas
                        .context
                        .rect(x as f64 * xmul, y as f64 * ymul, xmul, ymul);
                    self.canvas.context.clip();

                    // Foreground & text
                    self.canvas.context.set_fill_style_str(colors.0.as_str());
                    self.canvas.context.fill_text(
                        cell.symbol(),
                        x as f64 * xmul,
                        y as f64 * ymul,
                    )?;

                    // draw an underline if CursorShape::SteadyUnderScore was used
                    if let Some(pos) = self.cursor_position {
                        if pos.y as usize == y
                            && pos.x as usize == x
                            && cell.modifier.contains(Modifier::UNDERLINED)
                        {
                            self.canvas
                                .context
                                .fill_text("_", x as f64 * xmul, y as f64 * ymul)?;
                        }
                    }

                    self.canvas.context.restore();
                }
            }
        }
        self.canvas.context.translate(-5_f64, -5_f64)?;
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
            let line = &mut self.buffer[y];
            line.extend(std::iter::repeat_with(Cell::default).take(x.saturating_sub(line.len())));
            line[x] = cell.clone();
        }

        // Draw the cursor if set
        if let Some(pos) = self.cursor_position {
            let y = pos.y as usize;
            let x = pos.x as usize;
            let line = &mut self.buffer[y];
            if x < line.len() {
                let cursor_style = self.cursor_shape.show(line[x].style());
                line[x].set_style(cursor_style);
            }
        }

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
            self.prev_buffer = self.buffer.clone();
            self.initialized = true;
            return Ok(());
        }

        if self.buffer != self.prev_buffer {
            self.update_grid(false)?;
        }

        self.prev_buffer = self.buffer.clone();

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
        self.buffer = get_sized_buffer();
        Ok(())
    }

    fn size(&self) -> IoResult<Size> {
        Ok(Size::new(
            self.buffer[0].len().saturating_sub(1) as u16,
            self.buffer.len().saturating_sub(1) as u16,
        ))
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
