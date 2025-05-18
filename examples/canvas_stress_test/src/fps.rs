use compact_str::format_compact;
use ratzilla::ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};
use web_time::Instant;

/// Records and calculates frames per second.
///
/// `FpsRecorder` keeps track of frame timings in a ring buffer and
/// provides methods to calculate the current frames per second.
///
/// The ring buffer size is fixed at 16 frames, which provides a good
/// balance between responsiveness and stability in the FPS reading.
pub struct FpsRecorder {
    /// Current position in the ring buffer
    tail: usize,
    /// Ring buffer of frame timestamps. Length is a power of 2 for
    /// fast modulus operations.
    recorded_frame: [Instant; 16], // ^2 len far fast modulus
}

/// A widget for displaying FPS statistics.
///
/// `FpsStats` renders the current FPS value as text in a Ratatui buffer.
/// It supports customizing the style of both the label and the value.
///
/// This widget only fills the widget's required area and does not handle
/// layout or resizing.
pub struct FpsStats<'a> {
    /// Style for the "FPS: " label
    main_style: Style,
    /// Style for the numeric FPS value
    fps_value_style: Style,
    /// Reference to the FPS recorder that provides the data
    recorder: &'a FpsRecorder,
}

impl FpsRecorder {
    /// Creates a new FPS recorder.
    ///
    /// Initializes a ring buffer of 16 timestamps, all set to the current time.
    /// This means that it will take 16 frames before the FPS value stabilizes.
    pub fn new() -> Self {
        let recorder = Self {
            tail: 0,
            recorded_frame: [Instant::now(); 16],
        };

        debug_assert!(
            recorder.recorded_frame.len().is_power_of_two(),
            "recorded_frame length must be a power of two"
        );

        recorder
    }

    /// Records a new frame timestamp.
    ///
    /// Call this method once per frame to record the time at which the frame was
    /// rendered. The method updates the ring buffer and advances the tail position.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut recorder = FpsRecorder::new();
    ///
    /// // In render loop
    /// recorder.record();
    /// ```
    pub fn record(&mut self) {
        self.recorded_frame[self.tail] = Instant::now();
        self.tail = (self.tail + 1) & (self.recorded_frame.len() - 1);
    }

    /// Calculates the current frames per second.
    ///
    /// Returns the FPS based on how long it took to render the last 16 frames.
    /// The calculation measures the time between the oldest recorded frame and now,
    /// then divides the buffer size by this duration to get frames per second.
    ///
    /// # Returns
    ///
    /// The current frames per second as a floating point value.
    pub fn fps(&self) -> f32 {
        let elapsed = Instant::now()
            .duration_since(self.recorded_frame[self.tail])
            .as_secs_f32()
            .max(0.001); // avoid division by zero

        self.recorded_frame.len() as f32 / elapsed
    }
}

impl<'a> FpsStats<'a> {
    /// Creates a new FPS statistics widget.
    ///
    /// # Arguments
    ///
    /// * `recorder` - A reference to an `FpsRecorder` that provides the FPS data
    pub fn new(recorder: &'a FpsRecorder) -> Self {
        Self {
            main_style: Style::default(),
            fps_value_style: Style::default(),
            recorder,
        }
    }

    /// Sets the style for the "FPS: " label.
    ///
    /// # Arguments
    ///
    /// * `style` - The style to apply to the label
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn main_style(mut self, style: Style) -> Self {
        self.main_style = style;
        self
    }

    /// Sets the style for the numeric FPS value.
    ///
    /// # Arguments
    ///
    /// * `style` - The style to apply to the FPS value
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn fps_value_style(mut self, style: Style) -> Self {
        self.fps_value_style = style;
        self
    }
}

impl Widget for FpsStats<'_> {
    /// Renders the FPS widget to the provided buffer.
    ///
    /// Draws the "FPS: " label with the main style, followed by the
    /// numeric FPS value with the fps_value_style. The value is
    /// formatted to one decimal place with a fixed width.
    fn render(self, area: Rect, buf: &mut Buffer) {
        // draw the FPS label
        buf.set_string(area.x, area.y, "FPS: ", self.main_style);

        // draw the FPS value
        const FPS_OFFSET: u16 = "FPS: ".len() as u16;
        let fps = format_compact!("{:5.1}", self.recorder.fps());
        buf.set_string(area.x + FPS_OFFSET, area.y, fps, self.fps_value_style);
    }
}
