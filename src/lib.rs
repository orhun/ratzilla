use ratatui::prelude::Backend;
use ratatui::Frame;
use ratatui::Terminal;
use web_sys::window;

use std::cell::RefCell;
use std::rc::Rc;
use web_sys::wasm_bindgen::prelude::*;

mod utils;
mod wasm_backend;

pub use wasm_backend::WasmBackend;

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}

pub fn render_on_web<F>(mut terminal: Terminal<WasmBackend>, mut render_callback: F)
where
    F: FnMut(&mut Frame) + 'static,
{
    let cb = Rc::new(RefCell::new(None));
    *cb.borrow_mut() = Some(Closure::wrap(Box::new({
        let cb = cb.clone();
        move || {
            terminal.autoresize().unwrap();
            let mut frame = terminal.get_frame();
            render_callback(&mut frame);
            terminal.flush().unwrap();
            terminal.swap_buffers();
            terminal.backend_mut().flush().unwrap();
            request_animation_frame(cb.borrow().as_ref().unwrap());
        }
    }) as Box<dyn FnMut()>));
    request_animation_frame(cb.borrow().as_ref().unwrap());
}
