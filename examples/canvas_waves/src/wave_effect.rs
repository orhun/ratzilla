use ratzilla::ratatui::{buffer::Buffer, layout::Rect};
use std::{fmt::Debug, rc::Rc};
use tachyonfx::{
    color_from_hsl, default_shader_impl, CellFilter, CellIterator, ColorSpace, Duration,
    FilterProcessor, Interpolation, Shader,
};

/// A shader that creates wave interference patterns
#[derive(Debug, Clone)]
pub struct WaveInterference {
    /// Start time used to track elapsed time
    alive: Duration,
    waves: Vec<Wave>,
    /// Hue shift speed in degrees per second
    hue_shift_speed: f32,
    /// Optional rectangular area to apply the effect to
    area: Option<Rect>,
    /// Cell filter to control which cells are affected
    cell_filter: Option<FilterProcessor>,
    /// Color space to use for color calculations
    color_space: ColorSpace,
}

#[derive(Clone)]
struct Wave {
    amplitude: f32,
    f: Rc<Box<dyn Fn((f32, f32), f32) -> f32>>,
}

impl Wave {
    /// Creates a new wave with the given parameters
    pub fn new(amplitude: f32, f: impl Fn((f32, f32), f32) -> f32 + 'static) -> Self {
        Self {
            amplitude,
            f: Rc::new(Box::new(f)),
        }
    }

    pub fn calculate(&self, pos: (f32, f32), elapsed: f32) -> f32 {
        (self.f)(pos, elapsed) * self.amplitude
    }
}

impl WaveInterference {
    /// Creates a new wave interference effect with default settings
    pub fn new() -> Self {
        Self {
            alive: Duration::from_millis(0),
            waves: vec![
                Wave::new(1.3, wave_a),
                Wave::new(2.1, wave_b),
                Wave::new(0.9, wave_c),
                Wave::new(0.8, wave_d),
            ],
            hue_shift_speed: 30.0,
            area: None,
            cell_filter: None,
            color_space: ColorSpace::Hsl,
        }
    }

    fn calc_wave_amplitude(&self, pos: (f32, f32)) -> f32 {
        let elapsed = self.alive.as_secs_f32();

        // total amplitude of all waves
        let total_amplitude = self.waves.iter().map(|w| w.amplitude).sum::<f32>();

        self.waves
            .iter()
            .map(|w| w.calculate(pos, elapsed) * 0.5)
            .sum::<f32>()
            / total_amplitude
    }
}

impl Shader for WaveInterference {
    default_shader_impl!(area, filter, clone, color_space);

    fn name(&self) -> &'static str {
        "wave_interference"
    }

    fn process(&mut self, duration: Duration, buf: &mut Buffer, area: Rect) -> Option<Duration> {
        // Calculate elapsed time in seconds
        self.alive += duration;
        let elapsed = self.alive.as_secs_f32();

        let elapsed_cos = elapsed.cos();

        // NOTE: need to clone this because of signature and lifetime annotation change in
        // Shader::cell_iter in https://github.com/junkdog/tachyonfx/pull/42/files#diff-5671d25d4f836e2bf332bb7fb82756277aa49796ac1f32565a3615d58ea943df
        // apply effect to each cell in the area
        let filter_processor = self.filter_processor().cloned();
        let cell_iter = CellIterator::new(buf, area, filter_processor.as_ref());

        for (pos, cell) in cell_iter {
            let pos = (pos.x as f32, pos.y as f32);
            let normalized = self.calc_wave_amplitude(pos);
            assert!(
                normalized >= -1.0 && normalized <= 1.0,
                "Normalized value out of range: {}",
                normalized
            );

            let a = Interpolation::QuartOut.alpha(normalized.abs()) * normalized.signum();

            let hue_shift = elapsed * self.hue_shift_speed;
            let hue =
                (normalized * 360.0 + hue_shift + 1.4 * pos.0 - 3.2 * pos.1 * elapsed_cos) % 360.0;
            let lightness = 20.0 + (a * a * a.signum()) * 80.0;
            let saturation = 60.0 + a * 40.0;

            // clamp values to valid ranges
            let saturation = saturation.clamp(0.0, 100.0);
            let lightness = lightness.clamp(0.0, 100.0);

            // foreground and background colors
            cell.set_bg(color_from_hsl(
                (hue + 180.0) % 360.0,
                saturation,
                lightness * 1.0,
            ));
        }

        None
    }

    fn reset(&mut self) {
        self.alive = Duration::from_secs(0);
    }

    fn done(&self) -> bool {
        false
    }
}

fn wave_a(pos: (f32, f32), elapsed: f32) -> f32 {
    let (x, y) = (pos.0, pos.1);

    let a = (x * 0.1 - elapsed * 2.0).sin();
    let b = (y * 0.2 + elapsed * 1.0).cos();

    a * b
}

fn wave_b(pos: (f32, f32), elapsed: f32) -> f32 {
    let (x, y) = (pos.0, pos.1);

    let a = (x * 0.3 - elapsed * 1.5).cos();
    let b = (y * 0.1 - elapsed * 0.75).sin();

    (a + b) * 0.5
}

fn wave_c(pos: (f32, f32), elapsed: f32) -> f32 {
    let (x, y) = (pos.0, pos.1);

    let a = (x * 0.4 + elapsed * 1.0).cos();
    let b = (y * 0.75 + elapsed * 0.5).sin();
    a.max(b).powi(2)
}

fn wave_d(pos: (f32, f32), elapsed: f32) -> f32 {
    let y = pos.1;

    (y.sin() * 0.3 + elapsed).cos()
}

impl Debug for Wave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Wave")
            .field("amplitude", &self.amplitude)
            .finish()
    }
}
