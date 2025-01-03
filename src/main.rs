use gloo_render::request_animation_frame;
use gloo_render::AnimationFrame;
use js_sys::Function;
use ratatui::layout::Alignment;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;
use ratatui::Terminal;

use std::cell::RefCell;
use std::io::Result as IoResult;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

use web_sys::window;

mod utils;

mod wasm_backend;
use wasm_backend::WasmBackend;

struct App {
    count: u64,
    some_text: String,
}

impl App {
    fn request_animation_frame(f: &Closure<dyn FnMut()>) {
        window()
            .unwrap()
            .request_animation_frame(f.as_ref().unchecked_ref())
            .unwrap();
    }
}
// taken from https://github.com/rustwasm/gloo/blob/master/crates/render/src/lib.rs

fn render(terminal: &mut Terminal<WasmBackend>, app_state: &mut App) -> IoResult<()> {
    terminal
        .draw(|f| {
            web_sys::console::log_1(&"Drawing before".into());
            f.render_widget(
                Paragraph::new(format!("Count: {}", app_state.count))
                    .alignment(Alignment::Center)
                    .block(Block::bordered()),
                f.area(),
            );
            web_sys::console::log_1(&"Drawing".into());
        })
        .unwrap();

    web_sys::console::log_1(&"Yo, yo, yo".into());

    Ok(())
}

fn main() {
    let mut terminal = Terminal::new(WasmBackend::new()).unwrap();

    let mut app_state = App {
        count: 0,
        some_text: "Hello World".to_string(),
    };

    let cb = Rc::new(RefCell::new(None));

    *cb.borrow_mut() = Some(Closure::wrap(Box::new({
        let cb = cb.clone();
        move || {
            // This should repeat every frame
            app_state.count += 1;
            // render(&mut terminal, &mut app_state).unwrap();
            App::request_animation_frame(cb.borrow().as_ref().unwrap());
        }
    }) as Box<dyn FnMut()>));

    App::request_animation_frame(cb.borrow().as_ref().unwrap());
    // render(&mut terminal, &mut app_state).unwrap();
    web_sys::console::log_1(&"Done".into());
}
