// This example shows the full range of RGB colors that can be displayed in the browser.

mod wave_effect;

use ratzilla::{WebRenderer};
use tachyonfx::{EffectRenderer, IntoEffect};
use ratzilla::backend::webgl2::WebGl2BackendOptions;
use examples_shared::backend::{BackendType, MultiBackendBuilder};
use wave_effect::WaveInterference;

fn main() -> std::io::Result<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let (_backend_type, terminal) = MultiBackendBuilder::new(BackendType::WebGl2)
        .webgl2_options(WebGl2BackendOptions::new().measure_performance(true).grid_id("container"))
        .build_terminal()?;

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
