use ratatui::{prelude::Backend, Frame, Terminal};
use std::{cell::RefCell, rc::Rc};
use web_sys::{wasm_bindgen::prelude::*, window};

use crate::{
    error::Error,
    event::{KeyEvent, MouseEvent},
};

/// Trait for rendering on the web.
///
/// It provides all the necessary methods to render the terminal on the web
/// and also interact with the browser such as handling key and mouse events.
pub trait WebRenderer {
    /// Renders the terminal on the web.
    ///
    /// This method takes a closure that will be called on every update
    /// that the browser makes during [`requestAnimationFrame`] calls.
    ///
    /// TODO: Clarify and validate this.
    ///
    /// [`requestAnimationFrame`]: https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame
    fn draw_web<F>(self, render_callback: F)
    where
        F: FnMut(&mut Frame) + 'static;

    /// Handles key events.
    ///
    /// This method takes a closure that will be called on every `keydown`
    /// event. Calling this method multiple times will replace the previous
    /// key event handler.
    fn on_key_event<F>(&mut self, callback: F) -> Result<(), Error>
    where
        F: FnMut(KeyEvent) + 'static;

    /// Handles mouse events.
    ///
    /// This method takes a closure that will be called on mouse events
    /// (mousemove, mousedown, mouseup) with grid coordinates. Calling this
    /// method multiple times will replace the previous mouse event handler.
    ///
    /// Returns an error if the backend doesn't support mouse events (WebGL2).
    fn on_mouse_event<F>(&mut self, callback: F) -> Result<(), Error>
    where
        F: FnMut(MouseEvent) + 'static;

    /// Clears the current mouse event handler.
    fn clear_mouse_events(&mut self) -> Result<(), Error>;

    /// Clears the current key event handler.
    fn clear_key_events(&mut self) -> Result<(), Error>;

    /// Requests an animation frame.
    fn request_animation_frame(f: &Closure<dyn FnMut()>) {
        window()
            .unwrap()
            .request_animation_frame(f.as_ref().unchecked_ref())
            .unwrap();
    }
}

/// Implement [`WebRenderer`] for Ratatui's [`Terminal`].
///
/// This implementation creates a loop that calls the [`Terminal::draw`] method
/// and delegates event handling to backends that implement [`WebEventHandler`].
impl<T> WebRenderer for Terminal<T>
where
    T: Backend + WebEventHandler + 'static,
{
    fn draw_web<F>(mut self, mut render_callback: F)
    where
        F: FnMut(&mut Frame) + 'static,
    {
        let callback = Rc::new(RefCell::new(None));
        *callback.borrow_mut() = Some(Closure::wrap(Box::new({
            let cb = callback.clone();
            move || {
                self.draw(|frame| {
                    render_callback(frame);
                })
                .unwrap();
                Self::request_animation_frame(cb.borrow().as_ref().unwrap());
            }
        }) as Box<dyn FnMut()>));
        Self::request_animation_frame(callback.borrow().as_ref().unwrap());
    }

    fn on_key_event<F>(&mut self, callback: F) -> Result<(), Error>
    where
        F: FnMut(KeyEvent) + 'static,
    {
        self.backend_mut().setup_key_events(callback)
    }

    fn on_mouse_event<F>(&mut self, callback: F) -> Result<(), Error>
    where
        F: FnMut(MouseEvent) + 'static,
    {
        self.backend_mut().setup_mouse_events(callback)
    }

    fn clear_mouse_events(&mut self) -> Result<(), Error> {
        self.backend_mut().clear_mouse_events()
    }

    fn clear_key_events(&mut self) -> Result<(), Error> {
        self.backend_mut().clear_key_events()
    }
}

/// Trait for web event lifecycle management.
///
/// This trait provides event handling capabilities for web backends, including
/// proper setup and cleanup of event listeners to prevent memory leaks.
/// Each backend can implement its own event management strategy.
pub trait WebEventHandler {
    /// Sets up mouse event handling with proper cleanup of previous handlers.
    ///
    /// The callback will be called on mouse events with grid coordinates.
    /// Calling this multiple times will replace the previous handler cleanly.
    fn setup_mouse_events<F>(&mut self, callback: F) -> Result<(), Error>
    where
        F: FnMut(MouseEvent) + 'static;

    /// Clears any active mouse event handlers.
    fn clear_mouse_events(&mut self) -> Result<(), Error>;

    /// Sets up keyboard event handling with proper cleanup of previous handlers.
    ///
    /// The callback will be called on keydown events.
    /// Calling this multiple times will replace the previous handler cleanly.
    fn setup_key_events<F>(&mut self, callback: F) -> Result<(), Error>
    where
        F: FnMut(KeyEvent) + 'static;

    /// Clears any active key event handlers.
    fn clear_key_events(&mut self) -> Result<(), Error>;
}
