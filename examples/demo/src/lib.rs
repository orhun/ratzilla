//! # [Ratatui] Original Demo example
//!
//! The latest version of this example is available in the [examples] folder in the upstream.
//!
//! [Ratatui]: https://github.com/ratatui/ratatui
//! [examples]: https://github.com/ratatui/ratatui/blob/main/examples
//! [examples readme]: https://github.com/ratatui/ratatui/blob/main/examples/README.md

use std::{cell::RefCell, rc::Rc};

use app::App;
use clap::Parser;
use examples_shared::backend::{BackendType, MultiBackendBuilder};
use ratzilla::event::KeyCode;
use ratzilla::ratatui::style::Modifier;
use ratzilla::web_sys::wasm_bindgen::{self, prelude::*};
use ratzilla::WebRenderer;
use ratzilla::{backend::canvas::CanvasBackendOptions, backend::webgl2::WebGl2BackendOptions};

mod app;

mod effects;
mod ui;

/// Demo
#[derive(Debug, Parser)]
struct Cli {
    /// time in ms between two ticks.
    #[arg(short, long, default_value_t = 250)]
    tick_rate: u64,

    /// whether unicode symbols are used to improve the overall look of the app
    #[arg(short, long, default_value_t = true)]
    unicode: bool,
}

#[wasm_bindgen]
pub fn main() {
    console_error_panic_hook::set_once();

    let app_state = Rc::new(RefCell::new(App::new("Demo", false)));

    // Create backend with explicit size like main branch (1600x900)
    let canvas_options = CanvasBackendOptions::new()
        .font(String::from("16px Fira Code"))
        // Fira Code does not have an italic variation
        .disable_modifiers(Modifier::ITALIC);

    let webgl2_options = WebGl2BackendOptions::new()
        .measure_performance(true)
        .size((1600, 900));

    let terminal = MultiBackendBuilder::with_fallback(BackendType::WebGl2)
        .canvas_options(canvas_options)
        .webgl2_options(webgl2_options)
        .build_terminal()
        .unwrap();

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
                KeyCode::Char(' ') => {
                    app_state.pause_unpause();
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
}
