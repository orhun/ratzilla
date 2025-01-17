//! # [Ratatui] Original Demo example
//!
//! The latest version of this example is available in the [examples] folder in the repository.
//!
//! Please note that the examples are designed to be run against the `main` branch of the Github
//! repository. This means that you may not be able to compile with the latest release version on
//! crates.io, or the one that you have installed locally.
//!
//! See the [examples readme] for more information on finding examples that match the version of the
//! library you are using.
//!
//! [Ratatui]: https://github.com/ratatui/ratatui
//! [examples]: https://github.com/ratatui/ratatui/blob/main/examples
//! [examples readme]: https://github.com/ratatui/ratatui/blob/main/examples/README.md

use std::{cell::RefCell, error::Error, rc::Rc};

use app::App;
use clap::Parser;
use ratzilla::event::KeyCode;
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

fn main() -> Result<(), Box<dyn Error>> {
    let app_state = Rc::new(RefCell::new(App::new("Demo", false)));
    let backend = CanvasBackend::new().expect("Unable to create canvas backend");
    let terminal = Terminal::new(backend).unwrap();
    terminal.on_key_event({
        let app_state_cloned = app_state.clone();
        move |event| {
            let mut app_state = app_state_cloned.borrow_mut();
            match event.code {
                KeyCode::Tab => {
                    app_state.on_right();
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
