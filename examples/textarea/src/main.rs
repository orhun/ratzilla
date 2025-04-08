use std::{cell::RefCell, io, rc::Rc};

use ratatui::prelude::Stylize;
use ratzilla::{
    event::{KeyCode, KeyEvent},
    DomBackend, WebRenderer,
};

fn main() -> io::Result<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let backend = DomBackend::new()?;
    let terminal = ratatui::Terminal::new(backend)?;

    let app = Rc::new(RefCell::new(App::new()));

    terminal.on_key_event({
        let event_state = app.clone();
        move |key_event| {
            let mut state = event_state.borrow_mut();
            state.handle_events(key_event);
        }
    });

    terminal.draw_web({
        let render_state = app.clone();
        move |frame| {
            let state = render_state.borrow();
            state.render(frame);
        }
    });

    Ok(())
}

#[derive(Default)]
struct App<'a> {
    textarea: tui_textarea::TextArea<'a>,
    status_text: String,
}

impl<'a> App<'a> {
    fn new() -> Self {
        let mut textarea = tui_textarea::TextArea::default();
        textarea.set_block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Text Area Example"),
        );

        App {
            textarea,
            status_text: String::new(),
        }
    }

    fn render(&self, frame: &mut ratatui::Frame) {
        let chunks = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Min(0),
            ratatui::layout::Constraint::Length(3),
        ])
        .split(frame.area());

        let style = ratatui::style::Style::new().cyan().italic();
        let status = ratatui::text::Span::styled(self.status_text.as_str(), style);

        let status = ratatui::widgets::Paragraph::new(status)
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title("Status"),
            )
            .alignment(ratatui::layout::Alignment::Left);

        frame.render_widget(&self.textarea, chunks[0]);
        frame.render_widget(&status, chunks[1]);
    }

    fn handle_events(&mut self, key_event: KeyEvent) {
        self.status_text = std::format!("Last key pressed: {key_event:?}");

        if let Some(key) = try_convert_code(key_event.code) {
            self.textarea.input(Input {
                key,
                ctrl: key_event.ctrl,
                alt: key_event.alt,
                shift: key_event.shift,
            });
        }
    }
}


fn try_convert_code(code: KeyCode) -> Option<tui_textarea::Key> {
    match code {
        KeyCode::Char(c) => Some(tui_textarea::Key::Char(c)),
        KeyCode::F(n) => Some(tui_textarea::Key::F(n)),
        KeyCode::Backspace => Some(tui_textarea::Key::Backspace),
        KeyCode::Enter => Some(tui_textarea::Key::Enter),
        KeyCode::Left => Some(tui_textarea::Key::Left),
        KeyCode::Right => Some(tui_textarea::Key::Right),
        KeyCode::Up => Some(tui_textarea::Key::Up),
        KeyCode::Down => Some(tui_textarea::Key::Down),
        KeyCode::Tab => Some(tui_textarea::Key::Tab),
        KeyCode::Delete => Some(tui_textarea::Key::Delete),
        KeyCode::Home => Some(tui_textarea::Key::Home),
        KeyCode::End => Some(tui_textarea::Key::End),
        KeyCode::PageUp => Some(tui_textarea::Key::PageUp),
        KeyCode::PageDown => Some(tui_textarea::Key::PageDown),
        KeyCode::Esc => Some(tui_textarea::Key::Esc),
        KeyCode::Unidentified => None,
    }
}
