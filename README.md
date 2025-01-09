# Ratzilla

Build terminal-themed web applications with Rust and WebAssembly. Powered by [Ratatui].

## Quickstart

Add Ratzilla as a dependency in your `Cargo.toml`:

```sh
cargo add ratzilla
```

Here is a minimal example:

```rust
use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use ratzilla::{RenderOnWeb, WasmBackend};
use ratzilla::ratatui::{
    layout::Alignment,
    style::Color,
    widgets::{Block, Paragraph},
    Terminal,
};

fn main() -> io::Result<()> {
    let counter = Rc::new(RefCell::new(0));
    let backend = WasmBackend::new();
    let terminal = Terminal::new(backend)?;

    terminal.on_key_event({
        let counter_cloned = counter.clone();
        move |event| {
            if event == " " {
                let mut counter = counter_cloned.borrow_mut();
                *counter += 1;
            }
        }
    });

    terminal.render_on_web(move |f| {
        let counter = counter.borrow();
        f.render_widget(
            Paragraph::new(format!("Count: {counter}"))
                .alignment(Alignment::Center)
                .block(
                    Block::bordered()
                        .title("Ratzilla")
                        .title_alignment(Alignment::Center)
                        .border_style(Color::Yellow),
                ),
            f.area(),
        );
    });

    Ok(())
}
```

Ratzilla uses [trunk] to build and serve the web application.

Install trunk with:

```sh
cargo install --locked trunk
```

Then serve it on your browser:

```sh
trunk serve
```

Now go to `http://localhost:8080` and keep cooking!

[trunk]: https://trunkrs.dev
[Ratatui]: https://ratatui.rs
