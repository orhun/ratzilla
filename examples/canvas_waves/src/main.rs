// This example shows the full range of RGB colors that can be displayed in the browser.

mod wave_effect;

use ratzilla::{ratatui::Terminal, WebGl2Backend, WebRenderer};
use tachyonfx::{EffectRenderer, IntoEffect};
use ratzilla::backend::webgl2::WebGl2BackendOptions;
use wave_effect::WaveInterference;

fn main() -> std::io::Result<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let backend =
        WebGl2Backend::new_with_options(WebGl2BackendOptions::new().grid_id("container"))?;
    let terminal = Terminal::new(backend)?;

    let mut effect = WaveInterference::new().into_effect();
    let mut last_tick = web_time::Instant::now();

    terminal.draw_web(move |frame| {
        let now = web_time::Instant::now();
        let elapsed = now.duration_since(last_tick);
        last_tick = now;

        frame.render_effect(&mut effect, frame.area(), elapsed.into());
    });
    Ok(())
}
