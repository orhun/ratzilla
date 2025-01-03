use std::io::Result as IoResult;

use ratatui::buffer::Cell;
use ratatui::prelude::Backend;
use ratatui_core::backend::WindowSize;

use ratatui_core::layout::Position;
use ratatui_core::layout::Size;
use ratatui_core::style::Color;
use ratatui_core::style::Modifier;
use wasm_bindgen::JsValue;
use web_sys::window;
use web_sys::Document;
use web_sys::Element;

use crate::utils::create_span;

type TermSpan = ((Color, Color), Modifier, String);

#[derive(Debug)]
pub struct WasmBackend {
    buffer: Vec<Vec<Cell>>,
    // spans: Vec<Vec<TermSpan>>,
    grid: Element,
    document: Document,
}

impl WasmBackend {
    pub fn new() -> Self {
        // use this time to initialize the grid and the document object for the backend to use later on
        let window = window().unwrap();
        let document = window.document().unwrap();
        let div = document.create_element("div").unwrap();
        let body = document.body().unwrap();
        body.append_child(&div).unwrap();

        Self {
            buffer: get_sized_buffer(),

            grid: div,
            document,
        }
    }

    /// The rendering process is split into three steps.
    fn prerender(&mut self) {
        web_sys::console::log_1(&"hello from prerender".into());

        let mut grid: Vec<Element> = vec![];

        let Some(cell) = self.buffer.first().and_then(|l| l.first()) else {
            return;
        };

        let mut fg = cell.fg;
        let mut bg = cell.bg;
        let mut mods = cell.modifier;
        for line in self.buffer.iter() {
            let mut text = String::with_capacity(line.len());
            let mut line_buf: Vec<TermSpan> = Vec::new();
            for c in line {
                if fg != c.fg || bg != c.bg || mods != c.modifier {
                    // Create a new node, clear the text buffer, update the foreground/background
                    if !text.is_empty() {
                        let span = ((fg, bg), mods, text.to_owned());
                        line_buf.push(span);
                    }
                    mods = c.modifier;
                    fg = c.fg;
                    bg = c.bg;
                    text.clear();
                }
                text.push_str(c.symbol())
            }
            // Create a new node, combine into a `pre` tag, push onto buf
            if !text.is_empty() {
                line_buf.push(((fg, bg), mods, text.to_owned()));
            }
            web_sys::console::log_1(&text.clone().into());

            let elem = create_span(&self.document, &text, "color: rgb(255, 255, 255);");
            self.grid.append_child(&elem).unwrap();
            grid.push(elem);
        }
    }
}

impl Backend for WasmBackend {
    fn draw<'a, I>(&mut self, content: I) -> IoResult<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        web_sys::console::log_1(&"hello from draw".into());
        for (x, y, cell) in content {
            let y = y as usize;
            let x = x as usize;
            let line = &mut self.buffer[y];
            line.extend(std::iter::repeat_with(Cell::default).take(x.saturating_sub(line.len())));
            line[x] = cell.clone();
        }
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
        self.prerender();
        Ok(())
    }

    fn get_cursor_position(&mut self) -> IoResult<Position> {
        todo!()
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, _: P) -> IoResult<()> {
        todo!()
    }
}

/// Calculates the number of characters that can fit in the window.
pub fn get_window_size() -> (u16, u16) {
    let (w, h) = get_raw_window_size();
    // These are mildly magical numbers... make them more precise
    (w / 10, h / 20)
}

pub(crate) fn get_raw_window_size() -> (u16, u16) {
    fn js_val_to_int<I: TryFrom<usize>>(val: JsValue) -> Option<I> {
        val.as_f64().and_then(|i| I::try_from(i as usize).ok())
    }

    web_sys::window()
        .and_then(|s| {
            s.inner_width()
                .ok()
                .and_then(js_val_to_int::<u16>)
                .zip(s.inner_height().ok().and_then(js_val_to_int::<u16>))
        })
        .unwrap_or((120, 120))
}

// TODO: Improve this...
pub(crate) fn is_mobile() -> bool {
    get_raw_screen_size().0 < 550
}

/// Calculates the number of pixels that can fit in the window.
pub fn get_raw_screen_size() -> (i32, i32) {
    let s = web_sys::window().unwrap().screen().unwrap();
    (s.width().unwrap(), s.height().unwrap())
}

/// Calculates the number of characters that can fit in the window.
pub fn get_screen_size() -> (u16, u16) {
    let (w, h) = get_raw_screen_size();
    // These are mildly magical numbers... make them more precise
    (w as u16 / 10, h as u16 / 19)
}

fn get_sized_buffer() -> Vec<Vec<Cell>> {
    let (width, height) = if is_mobile() {
        get_screen_size()
    } else {
        get_window_size()
    };
    vec![vec![Cell::default(); width as usize]; height as usize]
}
