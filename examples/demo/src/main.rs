//! # [Ratatui] Original Demo example
//!
//! The latest version of this example is available in the [examples] folder in the upstream.
//!
//! [Ratatui]: https://github.com/ratatui/ratatui
//! [examples]: https://github.com/ratatui/ratatui/blob/main/examples
//! [examples readme]: https://github.com/ratatui/ratatui/blob/main/examples/README.md

use std::{cell::RefCell, io::Result, rc::Rc};

use app::App;
use clap::Parser;
use ratzilla::event::KeyCode;
use ratzilla::ratatui::style::Color;
use ratzilla::ratatui::Terminal;
use ratzilla::{CanvasBackend, RenderOnWeb};

mod app;

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

fn main() -> Result<()> {
    let app_state = Rc::new(RefCell::new(App::new("Demo", false)));
    let mut backend = CanvasBackend::new_with_size(1600, 900)?;
    backend.set_background_color(Color::Rgb(18, 18, 18));
    let terminal = Terminal::new(backend)?;
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
    terminal.render_on_web(move |f| {
        let mut app_state = app_state.borrow_mut();
        app_state.on_tick();
        ui::draw(f, &mut app_state);
    });

    Ok(())
}
