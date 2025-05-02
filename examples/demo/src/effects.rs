use ratzilla::ratatui::layout::{Constraint, Layout};
use ratzilla::ratatui::prelude::Color;
use tachyonfx::{fx::*, CellFilter, Duration, Effect, EffectTimer, Interpolation::*, Motion};

pub fn startup() -> Effect {
    let timer = EffectTimer::from_ms(3000, QuadIn);

    parallel(&[
        parallel(&[
            sweep_in(Motion::LeftToRight, 100, 20, Color::Black, timer),
            sweep_in(Motion::UpToDown, 100, 20, Color::Black, timer),
        ]),
        prolong_start(500, coalesce((2500, SineOut))),
    ])
}

pub(super) fn pulsate_selected_tab() -> Effect {
    let layout = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]);
    let highlighted_tab = CellFilter::AllOf(vec![
        CellFilter::Layout(layout, 0),
        CellFilter::FgColor(Color::LightYellow),
    ]);

    // never ends
    repeating(hsl_shift_fg([-70.0, 25.0, 30.0], (1000, SineInOut))).with_filter(highlighted_tab)
}

pub(super) fn change_tab() -> Effect {
    let layout = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]);
    let hsl_shift = [0.0, -100.0, -50.0];

    sequence(&[
        // close panel effect
        with_duration(
            Duration::from_millis(300),
            parallel(&[
                never_complete(dissolve((200, ExpoInOut))),
                never_complete(fade_to_fg(BG_COLOR, (200, BounceOut))),
            ]),
        ),
        // init pane, after having closed the (not) "old" one
        parallel(&[
            hsl_shift_fg(hsl_shift, (500, CircIn)).reversed(),
            fade_from(BG_COLOR, BG_COLOR, 200),
        ]),
    ])
    .with_filter(CellFilter::Layout(layout, 1))
}

const BG_COLOR: Color = Color::from_u32(0x121212);
