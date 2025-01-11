use std::io::Result as IoResult;

use ratatui::backend::WindowSize;
use ratatui::buffer::Cell;
use ratatui::layout::Position;
use ratatui::layout::Size;
use ratatui::prelude::Backend;
use ratatui::style::Styled;
use web_sys::js_sys::Boolean;
use web_sys::js_sys::Map;
use web_sys::wasm_bindgen::JsCast;
use web_sys::wasm_bindgen::JsValue;
use web_sys::window;
use web_sys::Document;
use web_sys::Element;

use crate::canvas_utils::*;
use crate::utils::*;

use crate::widgets::HYPERLINK;

#[derive(Debug)]
pub struct WasmCanvasBackend {
    initialized: bool,
    buffer: Vec<Vec<Cell>>,
    prev_buffer: Vec<Vec<Cell>>,
    ctx: web_sys::CanvasRenderingContext2d,
    canvas: Element,
    document: Document,
}

impl Default for WasmCanvasBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmCanvasBackend {
    pub fn new() -> Self {
        // use this time to initialize the grid and the document object for the backend to use later on
        let window = window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.create_element("canvas").unwrap();

        let canvas_ref: web_sys::HtmlCanvasElement = canvas
            .clone()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();
        canvas_ref.set_width(1400);
        canvas_ref.set_height(1000);

        let context_options = Map::new();
        context_options.set(&JsValue::from_str("alpha"), &Boolean::from(JsValue::TRUE));
        context_options.set(
            &JsValue::from_str("desynchronized"),
            &Boolean::from(JsValue::TRUE),
        );
        let ctx = canvas_ref
            .get_context_with_context_options("2d", &context_options)
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        ctx.set_font("16px monospace");
        ctx.set_text_baseline("top");

        let body = document.body().unwrap();
        body.append_child(&canvas).unwrap();

        Self {
            buffer: get_sized_buffer_canvas(&canvas_ref),
            prev_buffer: get_sized_buffer_canvas(&canvas_ref),
            canvas,
            document,
            ctx,

            initialized: false,
        }
    }

    // here's the deal, we compare the current buffer to the previous buffer and update only the cells that have changed since the last render call
    fn update_grid(&mut self, force_redraw: bool) {
        if force_redraw {
            self.ctx.clear_rect(
                0.0,
                0.0,
                self.canvas.client_width() as f64,
                self.canvas.client_height() as f64,
            );
        }

        let _ = self.ctx.translate(5 as f64, 5 as f64);
        let xmul = 10.0;
        let ymul = 19.0;
        for (y, line) in self.buffer.iter().enumerate() {
            for (x, cell) in line.iter().enumerate() {
                if cell != &self.prev_buffer[y][x] || force_redraw {
                    let colors = get_cell_color_canvas(&cell);

                    self.ctx.set_fill_style_str(colors.1.as_str());
                    let _ = self
                        .ctx
                        .fill_rect(x as f64 * xmul, y as f64 * ymul, xmul, ymul);

                    self.ctx.set_fill_style_str(colors.0.as_str());

                    let _ = self
                        .ctx
                        .fill_text(&cell.symbol(), x as f64 * xmul, y as f64 * ymul);
                }
            }
        }
        let _ = self.ctx.translate(-5 as f64, -5 as f64);
    }
}

impl Backend for WasmCanvasBackend {
    fn draw<'a, I>(&mut self, content: I) -> IoResult<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        // web_sys::console::log_1(&"hello from draw".into());
        for (x, y, cell) in content {
            let y = y as usize;
            let x = x as usize;
            let line = &mut self.buffer[y];
            line.extend(std::iter::repeat_with(Cell::default).take(x.saturating_sub(line.len())));
            line[x] = cell.clone();
        }
        // web_sys::console::log_1(&"hello from draw end ".into());
        Ok(())
    }

    fn hide_cursor(&mut self) -> IoResult<()> {
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
            self.buffer.first().unwrap().len().saturating_sub(1) as u16,
            self.buffer.len().saturating_sub(1) as u16,
        ))
    }

    fn window_size(&mut self) -> IoResult<WindowSize> {
        todo!()
    }

    fn flush(&mut self) -> IoResult<()> {
        if !self.initialized {
            self.update_grid(true);
            self.prev_buffer = self.buffer.clone();
            self.initialized = true;
            return Ok(());
        }
        if self.buffer != self.prev_buffer {
            self.update_grid(false);
        }
        self.prev_buffer = self.buffer.clone();
        Ok(())
    }

    fn get_cursor_position(&mut self) -> IoResult<Position> {
        todo!()
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, _: P) -> IoResult<()> {
        todo!()
    }
}
