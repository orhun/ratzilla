use std::{cell::RefCell, io, rc::Rc};

use ratzilla::ratatui::{
    layout::Alignment,
    style::Color,
    widgets::{Block, Paragraph},
};

use ratzilla::backend::canvas::CanvasBackendOptions;
use ratzilla::backend::dom::DomBackendOptions;
use ratzilla::backend::webgl2::WebGl2BackendOptions;
use ratzilla::{
    event::{KeyCode, MouseButton, MouseEvent, MouseEventKind},
    WebRenderer,
};

use examples_shared::backend::{BackendType, MultiBackendBuilder};

// Gruvbox bright orange color
const GRUVBOX_BRIGHT_ORANGE: Color = Color::Rgb(254, 128, 25);

fn main() -> io::Result<()> {
    let counter = Rc::new(RefCell::new(0));
    let mouse_position = Rc::new(RefCell::new((0, 0)));
    let mouse_button = Rc::new(RefCell::new(None::<MouseButton>));
    let mouse_event_kind = Rc::new(RefCell::new(None::<MouseEventKind>));

    // Create a shared mouse event handler closure
    let create_mouse_handler = || {
        let mouse_position_cloned = mouse_position.clone();
        let mouse_button_cloned = mouse_button.clone();
        let mouse_event_kind_cloned = mouse_event_kind.clone();
        move |mouse_event: MouseEvent| {
            let mut mouse_position = mouse_position_cloned.borrow_mut();
            *mouse_position = (mouse_event.col, mouse_event.row);
            let mut mouse_button = mouse_button_cloned.borrow_mut();
            *mouse_button = Some(mouse_event.button);
            let mut mouse_event_kind = mouse_event_kind_cloned.borrow_mut();
            *mouse_event_kind = Some(mouse_event.event);
        }
    };

    let dom_options = DomBackendOptions::default().mouse_event_handler(create_mouse_handler());

    let canvas_options = CanvasBackendOptions::new().mouse_event_handler(create_mouse_handler());

    let webgl2_options = WebGl2BackendOptions::new();

    let terminal = MultiBackendBuilder::with_fallback(BackendType::Dom)
        .dom_options(dom_options)
        .canvas_options(canvas_options)
        .webgl2_options(webgl2_options)
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
        let mouse_position = mouse_position.borrow();
        let mouse_button = mouse_button.borrow();
        let mouse_event_kind = mouse_event_kind.borrow();

        f.render_widget(
            Paragraph::new(format!(
                "Space pressed: {counter}\n\
                Column: {:?}\n\
                Row: {:?}\n\
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

        // Highlight the hovered cell
        let (hover_col, hover_row) = *mouse_position;
        let area = f.area();
        if hover_col < area.width && hover_row < area.height {
            if let Some(cell) = f
                .buffer_mut()
                .cell_mut((area.x + hover_col, area.y + hover_row))
            {
                cell.set_bg(GRUVBOX_BRIGHT_ORANGE);
            }
        }
    });

    Ok(())
}
