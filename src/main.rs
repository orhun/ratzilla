use ratatui::layout::Alignment;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;
use ratatui::Terminal;
use ratatui_core::backend::Backend;
use ratatui_core::backend::WindowSize;
use ratatui_core::buffer::Cell;
use ratatui_core::layout::Position;
use ratatui_core::layout::Size;
use ratatui_core::style::Color;
use ratatui_core::style::Modifier;
use std::cell::RefCell;
use std::io::Result as IoResult;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, Document, Element, HtmlElement};

type TermSpan = ((Color, Color), Modifier, String);

#[derive(Debug)]
struct WasmBackend {
    buffer: Vec<Vec<Cell>>,
    spans: Vec<Vec<TermSpan>>,
}

impl WasmBackend {
    pub fn new() -> Self {
        Self {
            buffer: get_sized_buffer(),
            spans: Vec::new(),
        }
    }

    /// The rendering process is split into three steps.
    fn prerender(&mut self) {
        let window = window().unwrap();
        let document = window.document().unwrap();
        let div = document.create_element("div").unwrap();
        div.set_attribute("id", "grid").unwrap();

        let mut grid: Vec<Element> = vec![];

        let Some(cell) = self.buffer.first().and_then(|l| l.first()) else {
            return;
        };

        let mut fg = cell.fg;
        let mut bg = cell.bg;
        let mut mods = cell.modifier;
        for line in self.buffer.iter() {
            let mut text = String::with_capacity(line.len());
            // let mut line_buf: Vec<TermSpan> = Vec::new();
            for c in line {
                // if fg != c.fg || bg != c.bg || mods != c.modifier {
                //     // Create a new node, clear the text buffer, update the foreground/background
                //     if !text.is_empty() {
                //         let span = ((fg, bg), mods, text.to_owned());
                //         line_buf.push(span);
                //     }
                //     mods = c.modifier;
                //     fg = c.fg;
                //     bg = c.bg;
                //     text.clear();
                // }
                text.push_str(c.symbol())
            }
            // Create a new node, combine into a `pre` tag, push onto buf
            // if !text.is_empty() {
            //     line_buf.push(((fg, bg), mods, text.to_owned()));
            // }
            web_sys::console::log_1(&text.clone().into());

            let elem = create_span(&document, &text, "color: rgb(255, 255, 255);");
            div.append_child(&elem).unwrap();
            grid.push(elem);
        }

        let body = document.body().unwrap();
        body.append_child(&div).unwrap();
    }
}

impl Backend for WasmBackend {
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

    fn hide_cursor(&mut self) -> IoResult<()> {
        Ok(())
    }

    fn show_cursor(&mut self) -> IoResult<()> {
        todo!()
    }

    fn get_cursor(&mut self) -> IoResult<(u16, u16)> {
        todo!()
    }

    fn set_cursor(&mut self, _x: u16, _y: u16) -> IoResult<()> {
        todo!()
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

fn create_span(document: &Document, text: &str, style: &str) -> Element {
    let span = document.create_element("span").unwrap();
    span.set_inner_html(text);
    span.set_attribute("style", style).unwrap();

    let pre = document.create_element("pre").unwrap();
    pre.set_attribute("style", "margin: 0px;").unwrap();
    pre.append_child(&span).unwrap();

    pre
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn render() -> Result<(), JsValue> {
    // Access the document
    let window = window().unwrap();
    let document = window.document().unwrap();
    let div = document.create_element("div").unwrap();
    div.set_attribute("id", "grid").unwrap();

    let mut terminal = Terminal::new(WasmBackend::new()).unwrap();

    terminal
        .draw(|f| {
            f.render_widget(
                Paragraph::new(f.count().to_string())
                    .alignment(Alignment::Center)
                    .block(Block::bordered()),
                f.area(),
            );
        })
        .unwrap();

    // for _i in 0..y {
    //     for _j in 0..x {
    //         let elem = create_span(&document, "A", "color: hsl(0, 100%, 50%);");
    //         div.append_child(&elem).unwrap();
    //         grid.push(elem);
    //     }
    //     div.append_child(&document.create_element("br").unwrap())
    //         .unwrap();
    // }

    // let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    // let g = f.clone();
    // *g.borrow_mut() = Some(Closure::new({
    //     let g = g.clone();
    //     move || {
    //         terminal
    //             .draw(|f| {
    //                 f.render_widget(
    //                     Paragraph::new(f.count().to_string())
    //                         .alignment(Alignment::Center)
    //                         .block(Block::bordered()),
    //                     f.area(),
    //                 );
    //             })
    //             .unwrap();
    //         window
    //             .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
    //             .unwrap();
    //     }
    // }));

    // request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

fn main() {
    render().unwrap();
}
