use std::cell::RefCell;
use std::rc::Rc;

use dom_test::render_on_web;
use dom_test::WasmBackend;
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::symbols::Marker;
use ratatui::widgets::canvas::Canvas;
use ratatui::widgets::canvas::Circle;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::Terminal;

mod utils;

struct App {
    count: u64,
    pub ball: Circle,
    vx: f64,
    vy: f64,
}

impl App {
    const fn new() -> Self {
        Self {
            count: 0,
            ball: Circle {
                x: 20.0,
                y: 20.0,
                radius: 5.0,
                color: Color::Red,
            },
            vx: 1.0,
            vy: 1.0,
        }
    }

    fn pong_canvas(&self) -> impl Widget + '_ {
        Canvas::default()
            .marker(Marker::Dot)
            .block(Block::bordered().title("Pong"))
            .paint(|ctx| {
                ctx.draw(&self.ball);
            })
            .x_bounds([0.0, 50.0])
            .y_bounds([0.0, 100.0])
    }

    fn update(&mut self) {
        if self.ball.x < 10.0 || self.ball.x > 40.0 {
            self.vx = -self.vx;
        }
        if self.ball.y < 10.0 || self.ball.y > 100.0 {
            self.vy = -self.vy;
        }
        self.ball.x += self.vx;
        self.ball.y += self.vy;
    }
}

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let app_state = Rc::new(RefCell::new(App::new()));
    let backend = WasmBackend::new();
    let app_state_cloned = app_state.clone();
    backend.on_key_event(move |event| {
        web_sys::console::log_1(&event.into());
        if event == "a" {
            app_state_cloned.borrow_mut().count = 0;
            app_state_cloned.borrow_mut().ball.color = Color::Green;
        } else if event == "b" {
            app_state_cloned.borrow_mut().ball.color = Color::Red;
        }
    });

    let terminal = Terminal::new(backend).unwrap();
    render_on_web(terminal, move |f| {
        app_state.borrow_mut().count += 1;
        app_state.borrow_mut().update();
        let horizontal =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let [left, right] = horizontal.areas(f.area());
        f.render_widget(
            Paragraph::new(format!("Count: {}", app_state.borrow().count))
                .alignment(Alignment::Center)
                .block(
                    Block::bordered()
                        .border_style(Style::default().fg(Color::Yellow).bg(Color::Black)),
                ),
            left,
        );
        f.render_widget(app_state.borrow().pong_canvas(), right);
    });

    web_sys::console::log_1(&"Done".into());
}
