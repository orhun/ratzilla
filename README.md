# Ratzilla

Build terminal-themed web applications with Rust and WebAssembly. Powered by [Ratatui].

## Quickstart

Add **Ratzilla** as a dependency in your `Cargo.toml`:

```sh
cargo add ratzilla
```

Here is a minimal example:

```rust no_run
use std::{cell::RefCell, io, rc::Rc};

use ratzilla::ratatui::{
    layout::Alignment,
    style::Color,
    widgets::{Block, Paragraph},
    Terminal,
};

use ratzilla::{event::KeyCode, DomBackend, WebRenderer};

fn main() -> io::Result<()> {
    let counter = Rc::new(RefCell::new(0));
    let backend = DomBackend::new()?;
    let terminal = Terminal::new(backend)?;

    terminal.on_key_event({
        let counter_cloned = counter.clone();
        move |key_event| {
            if key_event.code == KeyCode::Char(' ') {
                let mut counter = counter_cloned.borrow_mut();
                *counter += 1;
            }
        }
    });

    terminal.draw_web(move |f| {
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

Then add your `index.html` which imports the JavaScript module:

<details>
  <summary>index.html</summary>
  
```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta
      name="viewport"
      content="width=device-width, initial-scale=1.0, user-scalable=no"
    />
    <link
      rel="stylesheet"
      href="https://cdnjs.cloudflare.com/ajax/libs/firacode/6.2.0/fira_code.min.css"
    />
    <title>Ratzilla</title>
    <style>
      body {
        margin: 0;
        width: 100%;
        height: 100vh;
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        align-content: center;
        background-color: #121212;
      }
      pre {
        font-family: "Fira Code", monospace;
        font-size: 16px;
        margin: 0px;
      }
    </style>
  </head>
  <body>
    <script type="module">
      import init from "./pkg/ratzilla.js";
      init();
    </script>
  </body>
</html>
```

</details>

Install [trunk] to build and serve the web application.

```sh
cargo install --locked trunk
```

Then serve it on your browser:

```sh
trunk serve
```

Now go to `http://localhost:8080` and enjoy TUIs in your browser!

## Documentation

- [API Documentation](https://docs.rs/ratzilla)
- [Backends](https://docs.rs/ratzilla/latest/ratzilla/backend/index.html)
- [Widgets](https://docs.rs/ratzilla/latest/ratzilla/widgets/index.html)

## Examples

- [Minimal](https://github.com/orhun/ratzilla/tree/main/examples/minimal) ([Preview](https://orhun.dev/ratzilla/minimal))
- [Demo](https://github.com/orhun/ratzilla/tree/main/examples/demo) ([Preview](https://orhun.dev/ratzilla/demo))
- [Pong](https://github.com/orhun/ratzilla/tree/main/examples/pong) ([Preview](https://orhun.dev/ratzilla/pong))
- [Colors RGB](https://github.com/orhun/ratzilla/tree/main/examples/colors_rgb) ([Preview](https://orhun.dev/ratzilla/colors_rgb))
- [Animations](https://github.com/orhun/ratzilla/tree/main/examples/animations) ([Preview](https://orhun.dev/ratzilla/animations))
- [World Map](https://github.com/orhun/ratzilla/tree/main/examples/world_map) ([Preview](https://orhun.dev/ratzilla/world_map))

## Acknowledgements

Thanks to [Webatui] projects for the inspiration and the initial implementation of the essential parts of DOM backend.

Special thanks to [Martin Blasko] for his huge help and contributions.

Lastly, thanks to [Ratatui] for providing the core TUI components.

[trunk]: https://trunkrs.dev
[Ratatui]: https://ratatui.rs
[`DomBackend`]: https://docs.rs/ratzilla/latest/ratzilla/struct.DomBackend.html
[`CanvasBackend`]: https://docs.rs/ratzilla/latest/ratzilla/struct.CanvasBackend.html
[`Hyperlink`]: https://docs.rs/ratzilla/latest/ratzilla/widgets/struct.Hyperlink.html
[Webatui]: https://github.com/TylerBloom/webatui
[Martin Blasko]: https://github.com/MartinBspheroid

## Contributing

Pull requests are welcome!

Consider submitting your ideas via [issues](https://github.com/orhun/ratzilla/issues/new) first and check out the [existing issues](https://github.com/orhun/ratzilla/issues).

## License

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat&logo=GitHub&labelColor=1D272B&color=819188&logoColor=white)](./LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg?style=flat&logo=GitHub&labelColor=1D272B&color=819188&logoColor=white)](./LICENSE-APACHE)

Licensed under either of [Apache License Version 2.0](./LICENSE-APACHE) or [The MIT License](./LICENSE-MIT) at your option.

🦀 ノ( º \_ º ノ) - respect crables!

## Copyright

Copyright © 2025, [Orhun Parmaksız](mailto:orhunparmaksiz@gmail.com)
