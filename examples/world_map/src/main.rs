use std::io;

use ratzilla::ratatui::{
    symbols::Marker,
    widgets,
    widgets::canvas,
    style::Color,
    Terminal,
};

use ratzilla::{DomBackend, WebRenderer};

fn main() -> io::Result<()> {
    let backend = DomBackend::new()?;
    let terminal = Terminal::new(backend)?;

    terminal.draw_web(move |f| {
        let canvas = canvas::Canvas::default()
            .block(widgets::Block::bordered().title("ohai wurld!"))
            .marker(Marker::HalfBlock)
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0])
            .paint(|ctx| {
                ctx.draw(&canvas::Map {
                    resolution: canvas::MapResolution::High,
                    color: Color::Green,
                });
            });
        f.render_widget(canvas, f.area());
    });

    Ok(())
}
