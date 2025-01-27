use std::io;

use layout::Flex;
use ratzilla::{
    event::{KeyCode, KeyEvent},
    ratatui::{
        prelude::*,
        widgets::{Block, BorderType, Clear, Paragraph, Wrap},
    },
    utils::open_url,
    CanvasBackend, WebRenderer,
};
use tachyonfx::{
    fx, CenteredShrink, Duration, Effect, EffectRenderer, EffectTimer, Interpolation, Motion,
    Shader,
};

struct State {
    intro_effect: Effect,
    menu_effect: Effect,
}

impl Default for State {
    fn default() -> Self {
        Self {
            intro_effect: fx::sequence(&[
                fx::ping_pong(fx::sweep_in(
                    Motion::LeftToRight,
                    10,
                    0,
                    Color::Black,
                    EffectTimer::from_ms(3000, Interpolation::QuadIn),
                )),
                fx::coalesce((3000, Interpolation::SineOut)),
                fx::sleep(1000),
            ]),
            menu_effect: fx::sequence(&[
                fx::coalesce((3000, Interpolation::SineOut)),
                fx::sleep(1000),
            ]),
        }
    }
}

fn main() -> io::Result<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let backend = CanvasBackend::new()?;
    let terminal = Terminal::new(backend)?;
    let mut state = State::default();
    terminal.on_key_event(move |key| handle_key_event(key));
    terminal.draw_web(move |f| ui(f, &mut state));
    Ok(())
}

fn ui(f: &mut Frame<'_>, state: &mut State) {
    if state.intro_effect.running() {
        render_intro(f, state);
    } else {
        render_menu(f, state);
    }
}

fn handle_key_event(key: KeyEvent) {
    match key.code {
        KeyCode::Char('g') => {
            open_url("https://github.com/orhun/ratzilla", true).unwrap();
        }
        KeyCode::Char('d') => {
            open_url("https://orhun.dev/ratzilla/demo", false).unwrap();
        }
        _ => {}
    }
}

fn render_intro(f: &mut Frame<'_>, state: &mut State) {
    Clear.render(f.area(), f.buffer_mut());
    let area = f.area().inner_centered(25, 2);
    let main_text = Text::from(vec![
        Line::from("| R A T Z I L L A |"),
        Line::from("Stomping through the web"),
    ]);
    f.render_widget(main_text.light_green().centered(), area);
    f.render_effect(&mut state.intro_effect, area, Duration::from_millis(40));
}

fn render_menu(f: &mut Frame<'_>, state: &mut State) {
    let vertical = Layout::vertical([Constraint::Percentage(20)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(20)]).flex(Flex::Center);
    let [area] = vertical.areas(f.area());
    let [area] = horizontal.areas(area);

    let text = Text::from(vec![
        Line::default(),
        Line::from(vec![
            "[".into(),
            "g".light_green(),
            "] GitHub Repository".into(),
        ]),
        Line::from(vec!["[".into(), "d".light_green(), "] Demo".into()]),
    ]);

    f.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false })
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title(" Welcome to Ratzilla ")
                    .title_alignment(Alignment::Center),
            ),
        area,
    );
    f.render_effect(&mut state.menu_effect, area, Duration::from_millis(100));
}
