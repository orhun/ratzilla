use js_sys::Function;
use ratatui::layout::Alignment;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;
use ratatui::Terminal;

use std::io::Result as IoResult;

use wasm_bindgen::prelude::*;

use web_sys::window;

mod utils;

mod wasm_backend;
use wasm_backend::WasmBackend;

// taken from https://github.com/rustwasm/gloo/blob/master/crates/render/src/lib.rs

fn render() -> IoResult<()> {
    let mut terminal = Terminal::new(WasmBackend::new()).unwrap();

    terminal
        .draw(|f| {
            web_sys::console::log_1(&"Drawing before".into());
            f.render_widget(
                Paragraph::new(f.count().to_string())
                    .alignment(Alignment::Center)
                    .block(Block::bordered()),
                f.area(),
            );
            web_sys::console::log_1(&"Drawing".into());
        })
        .unwrap();

    web_sys::console::log_1(&"Yo, yo, yo".into());

    window()
        .unwrap()
        .request_animation_frame(&Function::from(Closure::once_into_js(move || {
            // render().unwrap();
        })))
        .unwrap();
    Ok(())
}

fn main() {
    render().unwrap();
    web_sys::console::log_1(&"Done".into());
}
