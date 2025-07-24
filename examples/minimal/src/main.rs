use std::{cell::RefCell, io, rc::Rc};

use ratzilla::ratatui::{
    layout::Alignment,
    style::Color,
    widgets::{Block, Paragraph}
};

use ratzilla::{event::KeyCode, WebRenderer};

use examples_shared::backend::{BackendType, MultiBackendBuilder};

fn main() -> io::Result<()> {
    let counter = Rc::new(RefCell::new(0));

    let mut terminal = MultiBackendBuilder::with_fallback(BackendType::Dom)
        .build_terminal()?;

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
