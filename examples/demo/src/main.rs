//! # [Ratatui] Original Demo example
//!
//! The latest version of this example is available in the [examples] folder in the upstream.
//!
//! [Ratatui]: https://github.com/ratatui/ratatui
//! [examples]: https://github.com/ratatui/ratatui/blob/main/examples
//! [examples readme]: https://github.com/ratatui/ratatui/blob/main/examples/README.md

use std::{cell::RefCell, io::Result, rc::Rc};

use app::App;
use examples_shared::backend::{BackendType, MultiBackendBuilder};
use ratzilla::event::KeyCode;
use ratzilla::WebRenderer;
use ratzilla::{backend::canvas::CanvasBackendOptions, backend::webgl2::WebGl2BackendOptions};

mod app;

mod effects;
mod ui;

fn main() -> Result<()> {
    let app_state = Rc::new(RefCell::new(App::new("Demo", true)));

    // Create backend with explicit size like main branch (1600x900)
    let canvas_options = CanvasBackendOptions::new().size((1600, 900));

    let webgl2_options = WebGl2BackendOptions::new()
        .measure_performance(true)
        .enable_console_debug_api()
        .enable_mouse_selection()
        .size((1600, 900));

    let terminal = MultiBackendBuilder::with_fallback(BackendType::WebGl2)
        .canvas_options(canvas_options)
        .webgl2_options(webgl2_options)
        .build_terminal()?;

    terminal.on_key_event({
        let app_state_cloned = app_state.clone();
        move |event| {
            let mut app_state = app_state_cloned.borrow_mut();
            match event.code {
                KeyCode::Right => {
                    app_state.on_right();
                }
                KeyCode::Left => {
                    app_state.on_left();
                }
                KeyCode::Up => {
                    app_state.on_up();
                }
                KeyCode::Down => {
                    app_state.on_down();
                }
                KeyCode::Char(c) => app_state.on_key(c),
                _ => {}
            }
        }
    });

    terminal.draw_web(move |f| {
        let mut app_state = app_state.borrow_mut();
        let elapsed = app_state.on_tick();
        ui::draw(elapsed, f, &mut app_state);
    });

    Ok(())
}
