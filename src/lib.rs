use ratatui::prelude::Backend;
use ratatui::Frame;
use ratatui::Terminal;
use web_sys::window;

use std::cell::RefCell;
use std::rc::Rc;
use web_sys::wasm_bindgen::prelude::*;

mod utils;
mod wasm_backend;
pub mod widgets;

pub use wasm_backend::WasmBackend;

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}

pub trait RenderOnWeb {
    fn render_on_web<F>(self, render_callback: F)
    where
        F: FnMut(&mut Frame) + 'static;
}

impl RenderOnWeb for Terminal<WasmBackend> {
    fn render_on_web<F>(mut self, mut render_callback: F)
    where
        F: FnMut(&mut Frame) + 'static,
    {
        let cb = Rc::new(RefCell::new(None));
        *cb.borrow_mut() = Some(Closure::wrap(Box::new({
            let cb = cb.clone();
            move || {
                self.autoresize().unwrap();
                let mut frame = self.get_frame();
                render_callback(&mut frame);
                self.flush().unwrap();
                self.swap_buffers();
                self.backend_mut().flush().unwrap();
                request_animation_frame(cb.borrow().as_ref().unwrap());
            }
        }) as Box<dyn FnMut()>));
        request_animation_frame(cb.borrow().as_ref().unwrap());
    }
}
