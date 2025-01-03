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

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::window;
mod utils;

mod wasm_backend;
use wasm_backend::WasmBackend;

struct App {
    count: u64,
    some_text: String,
    ball: Circle,
    vx: f64,
    vy: f64,
}

impl App {
    const fn new() -> Self {
        Self {
            count: 0,
            some_text: String::new(),
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
    fn request_animation_frame(f: &Closure<dyn FnMut()>) {
        window()
            .unwrap()
            .request_animation_frame(f.as_ref().unchecked_ref())
            .unwrap();
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
    let mut terminal = Terminal::new(WasmBackend::new()).unwrap();

    let mut app_state = App::new();

    let cb = Rc::new(RefCell::new(None));

    *cb.borrow_mut() = Some(Closure::wrap(Box::new({
        let cb = cb.clone();
        move || {
            // This should repeat every frame
            app_state.count += 1;
            app_state.update();
            terminal
                .draw(|f| {
                    let horizontal = Layout::horizontal([
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ]);
                    let vertical =
                        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]);
                    let [left, right] = horizontal.areas(f.area());
                    let [draw, map] = vertical.areas(left);
                    let [pong, boxes] = vertical.areas(right);

                    f.render_widget(
                        Paragraph::new(format!("Count: {}", app_state.count))
                            .alignment(Alignment::Center)
                            .block(
                                Block::bordered().border_style(
                                    Style::default().fg(Color::Yellow).bg(Color::Black),
                                ),
                            ),
                        left,
                    );
                    f.render_widget(app_state.pong_canvas(), right);
                    // web_sys::console::log_1(&"Drawing after".into());
                })
                .unwrap();

            App::request_animation_frame(cb.borrow().as_ref().unwrap());
        }
    }) as Box<dyn FnMut()>));

    App::request_animation_frame(cb.borrow().as_ref().unwrap());

    web_sys::console::log_1(&"Done".into());
}
