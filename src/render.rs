use ratatui::prelude::Backend;
use ratatui::Frame;
use ratatui::Terminal;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::wasm_bindgen::prelude::*;
use web_sys::window;

use crate::WasmBackend;

pub trait RenderOnWeb {
    fn render_on_web<F>(self, render_callback: F)
    where
        F: FnMut(&mut Frame) + 'static;

    fn on_key_event<F>(&self, mut callback: F)
    where
        F: FnMut(&str) + 'static,
    {
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
            web_sys::console::log_1(&event);
            callback(&event.key());
        });

        let window = window().unwrap();
        let document = window.document().unwrap();
        document
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    fn request_animation_frame(f: &Closure<dyn FnMut()>) {
        window()
            .unwrap()
            .request_animation_frame(f.as_ref().unchecked_ref())
            .unwrap();
    }
}

impl RenderOnWeb for Terminal<WasmBackend> {
    fn render_on_web<F>(mut self, mut render_callback: F)
    where
        F: FnMut(&mut Frame) + 'static,
    {
        let callback = Rc::new(RefCell::new(None));
        *callback.borrow_mut() = Some(Closure::wrap(Box::new({
            let cb = callback.clone();
            move || {
                self.autoresize().unwrap();
                let mut frame = self.get_frame();
                render_callback(&mut frame);
                self.flush().unwrap();
                self.swap_buffers();
                self.backend_mut().flush().unwrap();
                Self::request_animation_frame(cb.borrow().as_ref().unwrap());
            }
        }) as Box<dyn FnMut()>));
        Self::request_animation_frame(callback.borrow().as_ref().unwrap());
    }
}
