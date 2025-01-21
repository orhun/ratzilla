use std::io::Result as IoResult;

use ratatui::backend::WindowSize;
use ratatui::buffer::Cell;
use ratatui::layout::Position;
use ratatui::layout::Size;
use ratatui::prelude::Backend;
use web_sys::js_sys::Boolean;
use web_sys::js_sys::Map;
use web_sys::wasm_bindgen::JsCast;
use web_sys::wasm_bindgen::JsValue;
use web_sys::window;

use crate::backend::utils::*;
use crate::error::Error;

/// Canvas renderer.
#[derive(Debug)]
struct Canvas {
    /// Canvas element.
    inner: web_sys::HtmlCanvasElement,
    /// Rendering context.
    context: web_sys::CanvasRenderingContext2d,
}

impl Canvas {
    /// Constructs a new [`Canvas`].
    fn new(document: web_sys::Document) -> Result<Self, Error> {
        let element = document.create_element("canvas")?;
        let canvas = element
            .clone()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .expect("Unable to cast canvas element");

        canvas.set_width(1400);
        canvas.set_height(1000);

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
}

impl CanvasBackend {
    /// Constructs a new [`CanvasBackend`].
    pub fn new() -> Result<Self, Error> {
        let window = window().ok_or(Error::UnableToRetrieveWindow)?;
        let document = window.document().ok_or(Error::UnableToRetrieveDocument)?;
        let canvas = Canvas::new(document)?;
        let mut modifier_style = String::new();
        
        Ok(Self {
            buffer: get_sized_buffer_from_canvas(&canvas.inner),
            prev_buffer: get_sized_buffer_from_canvas(&canvas.inner),
            initialized: false,
            canvas,
        })
    }

    // Compare the current buffer to the previous buffer and updates the canvas accordingly.
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
                    let colors = get_cell_color_for_canvas(cell);

                    self.canvas.context.set_fill_style_str(colors.1.as_str());
                    self.canvas
                        .context
                        .fill_rect(x as f64 * xmul, y as f64 * ymul, xmul, ymul);

                    self.canvas.context.set_fill_style_str(colors.0.as_str());
                    self.canvas.context.fill_text(
                        cell.symbol(),
                        x as f64 * xmul,
                        y as f64 * ymul,
                    )?;
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

        Ok(())
    }

    fn hide_cursor(&mut self) -> IoResult<()> {Ok(())}
    fn show_cursor(&mut self) -> IoResult<()> {Ok(())}
    fn get_cursor(&mut self) -> IoResult<(u16, u16)> {Ok((0, 0))}
    fn set_cursor(&mut self, _x: u16, _y: u16) -> IoResult<()> {Ok(())}

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

    fn flush(&mut self) -> IoResult<()> {
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

    fn get_cursor_position(&mut self) -> IoResult<Position> {
        unimplemented!()
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, _: P) -> IoResult<()> {
        unimplemented!()
    }
}
