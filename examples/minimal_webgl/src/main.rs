use std::{cell::RefCell, io, rc::Rc};

use ratatui::layout::Position;
use ratzilla::ratatui::{
    layout::Alignment,
    style::Color,
    widgets::{Block, Paragraph},
    Terminal,
};

use ratzilla::{
    event::KeyCode, event::MouseButton, event::MouseEventKind, WebGl2Backend, WebRenderer,
};

fn main() -> io::Result<()> {
    let counter = Rc::new(RefCell::new(0));
    let mouse_position = Rc::new(RefCell::new((0, 0)));
    let mouse_button = Rc::new(RefCell::new(None::<MouseButton>));
    let mouse_event_kind = Rc::new(RefCell::new(None::<MouseEventKind>));

    let backend = WebGl2Backend::new()?;
    let terminal = Rc::new(Terminal::new(backend)?);

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
            Paragraph::new(format!(
                "Space pressed: {counter}\n\
                MouseX: {:?}\n\
                MouseY: {:?}\n\
                MouseButton: {mouse_button:?}\n\
                MouseEvent: {mouse_event_kind:?}",
                mouse_position.0, mouse_position.1
            ))
            .alignment(Alignment::Center)
            .block(
                Block::bordered()
                    .title("Ratzilla")
                    .title_alignment(Alignment::Center)
                    .border_style(Color::Yellow),
            ),
            f.area(),
        );

        let buf = f.buffer_mut();

        let pos = Position::new(mouse_position.0 as u16, mouse_position.1 as u16);

        if let Some(cell) = buf.cell_mut(pos.clone().to_owned()) {
            cell.set_bg(ratatui::style::Color::LightGreen);
        }
    });

    Ok(())
}
