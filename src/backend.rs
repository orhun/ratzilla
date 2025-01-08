use std::io::Result as IoResult;

use ratatui::backend::WindowSize;
use ratatui::buffer::Cell;
use ratatui::layout::Position;
use ratatui::layout::Size;
use ratatui::prelude::Backend;
use web_sys::wasm_bindgen::prelude::Closure;
use web_sys::wasm_bindgen::JsCast;
use web_sys::window;
use web_sys::Document;
use web_sys::Element;

use crate::utils::*;
use crate::widgets::HYPERLINK;

#[derive(Debug)]
pub struct WasmBackend {
    initialized: bool,
    buffer: Vec<Vec<Cell>>,
    prev_buffer: Vec<Vec<Cell>>,
    cells: Vec<Element>,
    grid: Element,
    document: Document,
}

impl WasmBackend {
    pub fn new() -> Self {
        // use this time to initialize the grid and the document object for the backend to use later on
        let window = window().unwrap();
        let document = window.document().unwrap();
        let div = document.create_element("div").unwrap();
        div.set_attribute("id", "grid").unwrap();
        let body = document.body().unwrap();
        body.append_child(&div).unwrap();

        Self {
            buffer: get_sized_buffer(),
            prev_buffer: get_sized_buffer(),
            grid: div,
            document,
            cells: vec![],
            initialized: false,
        }
    }

    fn prerender(&mut self) {
        web_sys::console::log_1(&"hello from prerender".into());

        for line in self.buffer.iter() {
            let mut line_cells: Vec<Element> = Vec::new();
            let mut hyperlink: Vec<Cell> = Vec::new();
            for (i, cell) in line.iter().enumerate() {
                if cell.modifier.contains(HYPERLINK) {
                    hyperlink.push(cell.clone());
                    // If the next cell is not part of the hyperlink, close it
                    if !line
                        .get(i + 1)
                        .map(|c| c.modifier.contains(HYPERLINK))
                        .unwrap_or(false)
                    {
                        let anchor = create_anchor(&self.document, &hyperlink);
                        for link_cell in &hyperlink {
                            let span = create_span(&self.document, link_cell);
                            self.cells.push(span.clone());
                            anchor.append_child(&span).unwrap();
                        }
                        line_cells.push(anchor);
                        hyperlink.clear();
                    }
                } else {
                    let span = create_span(&self.document, cell);
                    self.cells.push(span.clone());
                    line_cells.push(span);
                }
            }

            // Create a <pre> element for the line
            let pre = self.document.create_element("pre").unwrap();
            pre.set_attribute("style", "margin: 0px;").unwrap();

            // Append all elements (spans and anchors) to the <pre>
            for elem in line_cells {
                pre.append_child(&elem).unwrap();
            }

            // Append the <pre> to the grid
            self.grid.append_child(&pre).unwrap();
        }
    }

    // here's the deal, we compare the current buffer to the previous buffer and update only the cells that have changed since the last render call
    fn update_grid(&mut self) {
        for (y, line) in self.buffer.iter().enumerate() {
            for (x, cell) in line.iter().enumerate() {
                if cell.modifier.contains(HYPERLINK) {
                    continue;
                }
                if cell != &self.prev_buffer[y][x] {
                    // web_sys::console::log_1(&format!("Cell different at ({}, {})", x, y).into());
                    let elem = self.cells[y * self.buffer[0].len() + x].clone();
                    // web_sys::console::log_1(&"Element retrieved".into());
                    elem.set_inner_html(&cell.symbol());
                    elem.set_attribute("style", &get_cell_color(cell)).unwrap();
                    // web_sys::console::log_1(&"Inner HTML set".into());
                }
            }
        }
    }

    pub fn on_key_event<F>(&self, mut callback: F)
    where
        F: FnMut(&str) + 'static,
    {
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
            web_sys::console::log_1(&event);
            callback(&event.key());
        });
        self.document
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }
}

impl Backend for WasmBackend {
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
            // web_sys::console::log_1(&"hello from flush".into());
            self.prerender();
            self.prev_buffer = self.buffer.clone(); // set the previous buffer to the current buffer for the first render
            self.initialized = true;
        }
        // web_sys::console::log_1(&"flush1".into());
        // check if the buffer has changed since the last render and update the grid
        if self.buffer != self.prev_buffer {
            self.update_grid();
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
