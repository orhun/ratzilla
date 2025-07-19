// This example shows the full range of RGB colors that can be displayed in the browser.

use palette::{convert::FromColorUnclamped, Okhsv, Srgb};
use ratzilla::{
    ratatui::{
        buffer::Buffer,
        layout::{Position, Rect},
        style::Color,
        widgets::Widget,
        Terminal,
    },
    WebRenderer,
};
use examples_shared::{BackendType};
use examples_shared::backend::multi_backend_builder;

fn main() -> std::io::Result<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let (_backend_type, terminal) = multi_backend_builder(BackendType::Canvas)
        .build_terminal()?;
    let mut app = ColorsWidget::default();
    terminal.draw_web(move |frame| {
        frame.render_widget(&mut app, frame.area());
    });
    Ok(())
}

/// A widget that displays the full range of RGB colors that can be displayed in the terminal.
///
/// This widget is animated and will change colors over time.
#[derive(Debug, Default)]
struct ColorsWidget {
    /// The colors to render - should be double the height of the area as we render two rows of
    /// pixels for each row of the widget using the half block character. This is computed any time
    /// the size of the widget changes.
    colors: Vec<Vec<Color>>,

    /// the number of elapsed frames that have passed - used to animate the colors by shifting the
    /// x index by the frame number
    frame_count: usize,
}

/// Widget impl for `ColorsWidget`
///
/// This is implemented on a mutable reference so that we can update the frame count and store a
/// cached version of the colors to render instead of recalculating them every frame.
impl Widget for &mut ColorsWidget {
    /// Render the widget
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.setup_colors(area);
        let colors = &self.colors;
        for (xi, x) in (area.left()..area.right()).enumerate() {
            // animate the colors by shifting the x index by the frame number
            let xi = (xi + self.frame_count) % (area.width as usize);
            for (yi, y) in (area.top()..area.bottom()).enumerate() {
                // render a half block character for each row of pixels with the foreground color
                // set to the color of the pixel and the background color set to the color of the
                // pixel below it
                let fg = colors[yi * 2][xi];
                let bg = colors[yi * 2 + 1][xi];
                buf[Position::new(x, y)].set_char('▀').set_fg(fg).set_bg(bg);
            }
        }
        self.frame_count += 1;
    }
}

impl ColorsWidget {
    /// Setup the colors to render.
    ///
    /// This is called once per frame to setup the colors to render. It caches the colors so that
    /// they don't need to be recalculated every frame.
    #[allow(clippy::cast_precision_loss)]
    fn setup_colors(&mut self, size: Rect) {
        let Rect { width, height, .. } = size;
        // double the height because each screen row has two rows of half block pixels
        let height = height as usize * 2;
        let width = width as usize;
        // only update the colors if the size has changed since the last time we rendered
        if self.colors.len() == height && self.colors[0].len() == width {
            return;
        }
        self.colors = Vec::with_capacity(height);
        for y in 0..height {
            let mut row = Vec::with_capacity(width);
            for x in 0..width {
                let hue = x as f32 * 360.0 / width as f32;
                let value = (height - y) as f32 / height as f32;
                let saturation = Okhsv::max_saturation();
                let color = Okhsv::new(hue, saturation, value);
                let color = Srgb::<f32>::from_color_unclamped(color);
                let color: Srgb<u8> = color.into_format();
                let color = Color::Rgb(color.red, color.green, color.blue);
                row.push(color);
            }
            self.colors.push(row);
        }
    }
}
