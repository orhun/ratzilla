use std::{cell::RefCell, io, rc::Rc};

use ratzilla::ratatui::{
    layout::Alignment,
    style::Color,
    widgets::{Block, Paragraph},
    Terminal,
};

use ratzilla::{event::KeyCode, event::MouseEventKind, event::MouseButton, DomBackend, WebRenderer};

fn main() -> io::Result<()> {
    let counter = Rc::new(RefCell::new(0));
    let mouse_position = Rc::new(RefCell::new((0, 0)));
    let mouse_button = Rc::new(RefCell::new(None::<MouseButton>));
    let mouse_event_kind = Rc::new(RefCell::new(None::<MouseEventKind>));
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

    terminal.on_mouse_event({
        let mouse_position_cloned = mouse_position.clone();
        let mouse_button_cloned = mouse_button.clone();
        let mouse_event_kind_cloned = mouse_event_kind.clone();
        move |mouse_event| {
            let mut mouse_position = mouse_position_cloned.borrow_mut();
            *mouse_position = (mouse_event.x, mouse_event.y);
            let mut mouse_button = mouse_button_cloned.borrow_mut();
            *mouse_button = Some(mouse_event.button);
            let mut mouse_event_kind = mouse_event_kind_cloned.borrow_mut();
            *mouse_event_kind = Some(mouse_event.event);
        }
    });

    terminal.draw_web(move |f| {
        let counter = counter.borrow();
        let mouse_position = mouse_position.borrow();
        let mouse_button = mouse_button.borrow();
        let mouse_event_kind = mouse_event_kind.borrow();

        f.render_widget(
            Paragraph::new(format!("Count: {counter}\nMouseX: {:?}, MouseY: {:?}\nMouseButton: {:?}\nMouseEvent: {:?}", mouse_position.0, mouse_position.1, mouse_button, mouse_event_kind))
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
