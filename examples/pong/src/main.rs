use std::cell::RefCell;
use std::rc::Rc;

use dom_test::utils::set_document_title;
use dom_test::widgets::Hyperlink;
use dom_test::RenderOnWeb;
use dom_test::WasmBackend;

use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::symbols::Marker;
use ratatui::widgets::canvas::Canvas;
use ratatui::widgets::canvas::Circle;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::Terminal;

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
                color: Color::Green,
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
    backend.on_key_event({
        let app_state_cloned = app_state.clone();
        move |event| {
            let mut app_state = app_state_cloned.borrow_mut();
            if event == "q" {
                set_document_title("Grind to win");
            }
            if event == "r" {
                set_document_title("RATATUI ! ! !");
            }
            if event == "a" {
                app_state.count = 0;
                app_state.ball.color = Color::Green;
            } else if event == "b" {
                app_state.ball.color = Color::Red;
            }
        }
    });

    let terminal = Terminal::new(backend).unwrap();
    terminal.render_on_web(move |f| {
        let mut app_state = app_state.borrow_mut();
        app_state.count += 1;
        app_state.update();
        let horizontal =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let [left, right] = horizontal.areas(f.area());

        f.render_widget(
            Paragraph::new(format!("Count: {}", app_state.count))
                .alignment(Alignment::Center)
                .block(
                    Block::bordered()
                        .border_style(Style::default().fg(Color::Yellow).bg(Color::Black)),
                ),
            left,
        );
        f.render_widget(app_state.pong_canvas(), right);

        let link = Hyperlink::new("https://orhun.dev");
        f.render_widget(link, Rect::new(75, 10, 20, 1));
    });
}
