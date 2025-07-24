use std::{cell::RefCell, io, rc::Rc};

use ratzilla::ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::Color,
    widgets::{Block, Paragraph},
};

use ratzilla::{
    backend::{canvas::CanvasBackendOptions, dom::DomBackendOptions, webgl2::WebGl2BackendOptions},
    event::{KeyCode, MouseEvent},
    WebRenderer,
};

use examples_shared::backend::{BackendType, MultiBackendBuilder};

// Gruvbox bright orange color
const GRUVBOX_BRIGHT_ORANGE: Color = Color::Rgb(254, 128, 25);

fn main() -> io::Result<()> {
    let counter = Rc::new(RefCell::new(0));
    let mouse_position = Rc::new(RefCell::new((0, 0)));
    let mouse_event_data = Rc::new(RefCell::new(None::<MouseEvent>));

    let mut terminal = MultiBackendBuilder::with_fallback(BackendType::Dom)
        .dom_options(DomBackendOptions::default())
        .canvas_options(CanvasBackendOptions::new())
        .webgl2_options(WebGl2BackendOptions::new())
        .build_terminal()?;

    // Set up mouse event handling using the new WebRenderer API
    terminal
        .on_mouse_event({
            let mouse_position_cloned = mouse_position.clone();
            let mouse_event_data_cloned = mouse_event_data.clone();
            move |mouse_event: MouseEvent| {
                let mut mouse_position = mouse_position_cloned.borrow_mut();
                *mouse_position = (mouse_event.col, mouse_event.row);
                let mut mouse_event_data = mouse_event_data_cloned.borrow_mut();
                *mouse_event_data = Some(mouse_event);
            }
        })
        .ok(); // WebGL2 backend doesn't support mouse events, so we ignore the error

    terminal
        .on_key_event({
            let counter_cloned = counter.clone();
            move |key_event| {
                if key_event.code == KeyCode::Char(' ') {
                    let mut counter = counter_cloned.borrow_mut();
                    *counter += 1;
                }
            }
        })
        .ok(); // Ignore errors for consistency

    terminal.draw_web(move |f| {
        let counter = counter.borrow();
        let mouse_position = mouse_position.borrow();
        let mouse_event_data = mouse_event_data.borrow();

        // Split the area into sections
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Space for counter
                Constraint::Min(1),    // Rest for mouse event
            ])
            .split(f.area());

        // Render counter (centered)
        f.render_widget(
            Paragraph::new(format!("Space pressed: {counter}"))
                .alignment(Alignment::Center)
                .block(
                    Block::bordered()
                        .title("Ratzilla")
                        .title_alignment(Alignment::Center)
                        .border_style(Color::Yellow),
                ),
            layout[0],
        );

        // Render mouse event (left-aligned within centered box)
        f.render_widget(
            Paragraph::new(
                mouse_event_data
                    .as_ref()
                    .map(|e| format!("{:#?}", e))
                    .unwrap_or_else(|| "No mouse events yet".to_string()),
            )
            .alignment(Alignment::Left)
            .block(
                Block::bordered()
                    .title("Mouse Event")
                    .title_alignment(Alignment::Center)
                    .border_style(Color::Yellow),
            ),
            layout[1],
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
