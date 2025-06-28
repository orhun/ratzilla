use ratzilla::{
    backend::{canvas::CanvasBackendOptions, dom::DomBackendOptions, webgl2::WebGl2BackendOptions},
    ratatui::{prelude::Backend, Terminal, TerminalOptions},
    CanvasBackend, DomBackend, WebGl2Backend,
};
use std::{cell::RefCell, io};
use web_sys::{wasm_bindgen::JsValue, Url};
use web_time::Instant;

/// Records and calculates frames per second.
///
/// `FpsRecorder` keeps track of frame timings in a ring buffer and
/// provides methods to calculate the current frames per second.
pub struct FpsRecorder {
    /// Current position in the ring buffer
    tail: usize,
    /// Ring buffer of frame timestamps. Length is a power of 2 for
    /// fast modulus operations.
    recorded_frame: [Instant; 16],
}

impl FpsRecorder {
    /// Creates a new FPS recorder.
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
    pub fn record(&mut self) {
        self.recorded_frame[self.tail] = Instant::now();
        self.tail = (self.tail + 1) & (self.recorded_frame.len() - 1);
    }

    /// Calculates the current frames per second.
    pub fn fps(&self) -> f32 {
        // Find the newest recorded timestamp (the one just before tail)
        let newest_idx = if self.tail == 0 {
            self.recorded_frame.len() - 1
        } else {
            self.tail - 1
        };

        let elapsed = self.recorded_frame[newest_idx]
            .duration_since(self.recorded_frame[self.tail])
            .as_secs_f32()
            .max(0.001); // avoid division by zero

        // We have 16 frames, so there are 15 intervals between them
        (self.recorded_frame.len() - 1) as f32 / elapsed
    }
}

use std::thread_local;

thread_local! {
    /// Thread-local FPS recorder instance for shared use across examples
    static FPS_RECORDER: RefCell<Option<FpsRecorder>> = RefCell::new(None);
}

/// Initialize the global FPS recorder
pub fn init_fps_recorder() {
    FPS_RECORDER.with(|recorder| {
        *recorder.borrow_mut() = Some(FpsRecorder::new());
    });
}

/// Record a frame for FPS calculation
pub fn record_frame() {
    FPS_RECORDER.with(|recorder| {
        if let Some(ref mut fps_recorder) = *recorder.borrow_mut() {
            fps_recorder.record();
            // Update the footer FPS display
            let fps = fps_recorder.fps();
            update_fps_display(fps);
        }
    });
}

/// Get the current FPS value
pub fn get_current_fps() -> f32 {
    FPS_RECORDER.with(|recorder| {
        if let Some(ref fps_recorder) = *recorder.borrow() {
            fps_recorder.fps()
        } else {
            0.0
        }
    })
}

/// Update the FPS display in the footer
fn update_fps_display(fps: f32) {
    let _ = (|| -> Result<(), JsValue> {
        let window = web_sys::window().ok_or("No window")?;
        let document = window.document().ok_or("No document")?;

        if let Some(fps_element) = document.get_element_by_id("ratzilla-fps") {
            fps_element.set_text_content(Some(&format!("{:.1}", fps)));
        }

        Ok(())
    })();
}

#[derive(Debug, Default)]
pub struct MultiBackendBuilder {
    default_backend: BackendType,
    terminal_options: TerminalOptions,
    canvas_options: CanvasBackendOptions,
    dom_options: DomBackendOptions,
    webgl2_options: WebGl2BackendOptions,
}

impl MultiBackendBuilder {
    fn new(default_backend: BackendType) -> Self {
        Self {
            default_backend,
            ..Self::default()
        }
    }

    pub fn terminal_options(mut self, options: TerminalOptions) -> Self {
        self.terminal_options = options;
        self
    }

    /// Set options for the Canvas backend
    pub fn canvas_options(mut self, options: CanvasBackendOptions) -> Self {
        self.canvas_options = options;
        self
    }

    /// Set options for the DOM backend
    pub fn dom_options(mut self, options: DomBackendOptions) -> Self {
        self.dom_options = options;
        self
    }

    /// Set options for the WebGL2 backend
    pub fn webgl2_options(mut self, options: WebGl2BackendOptions) -> Self {
        self.webgl2_options = options;
        self
    }

    pub fn build_terminal(self) -> io::Result<(BackendType, Terminal<FpsTrackingBackend>)> {
        let backend_type = get_backend_from_query(self.default_backend);
        let backend = create_backend_with_options(
            backend_type,
            Some(self.dom_options),
            Some(self.canvas_options),
            Some(self.webgl2_options),
        )?;

        // Initialize FPS recorder
        init_fps_recorder();

        // Wrap backend with FPS tracking
        let fps_backend = FpsTrackingBackend::new(backend);
        let terminal = Terminal::with_options(fps_backend, self.terminal_options)?;

        // Inject footer (ignore errors)
        let _ = inject_backend_footer(backend_type);

        Ok((backend_type, terminal))
    }
}

/// Create a new MultiBackendBuilder with the default backend type
pub fn backend_from_query_param(default: BackendType) -> MultiBackendBuilder {
    MultiBackendBuilder::new(default)
}

/// Available backend types
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum BackendType {
    #[default]
    Dom,
    Canvas,
    WebGl2,
}

impl BackendType {
    /// Get the backend type from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "dom" => Some(BackendType::Dom),
            "canvas" => Some(BackendType::Canvas),
            "webgl2" => Some(BackendType::WebGl2),
            _ => None,
        }
    }

    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            BackendType::Dom => "dom",
            BackendType::Canvas => "canvas",
            BackendType::WebGl2 => "webgl2",
        }
    }

    /// Get a human-readable name
    pub fn display_name(&self) -> &'static str {
        match self {
            BackendType::Dom => "DOM",
            BackendType::Canvas => "Canvas",
            BackendType::WebGl2 => "WebGL2",
        }
    }
}

/// Enum wrapper for different backends
pub enum RatzillaBackend {
    Dom(DomBackend),
    Canvas(CanvasBackend),
    WebGl2(WebGl2Backend),
}

/// Backend wrapper that automatically tracks FPS
pub struct FpsTrackingBackend {
    inner: RatzillaBackend,
}

impl FpsTrackingBackend {
    pub fn new(backend: RatzillaBackend) -> Self {
        Self { inner: backend }
    }
}

impl Backend for RatzillaBackend {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a ratzilla::ratatui::buffer::Cell)>,
    {
        match self {
            RatzillaBackend::Dom(backend) => backend.draw(content),
            RatzillaBackend::Canvas(backend) => backend.draw(content),
            RatzillaBackend::WebGl2(backend) => backend.draw(content),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            RatzillaBackend::Dom(backend) => backend.flush(),
            RatzillaBackend::Canvas(backend) => backend.flush(),
            RatzillaBackend::WebGl2(backend) => backend.flush(),
        }
    }

    fn size(&self) -> io::Result<ratzilla::ratatui::layout::Size> {
        match self {
            RatzillaBackend::Dom(backend) => backend.size(),
            RatzillaBackend::Canvas(backend) => backend.size(),
            RatzillaBackend::WebGl2(backend) => backend.size(),
        }
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        match self {
            RatzillaBackend::Dom(backend) => backend.hide_cursor(),
            RatzillaBackend::Canvas(backend) => backend.hide_cursor(),
            RatzillaBackend::WebGl2(backend) => backend.hide_cursor(),
        }
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        match self {
            RatzillaBackend::Dom(backend) => backend.show_cursor(),
            RatzillaBackend::Canvas(backend) => backend.show_cursor(),
            RatzillaBackend::WebGl2(backend) => backend.show_cursor(),
        }
    }

    fn get_cursor_position(&mut self) -> io::Result<ratzilla::ratatui::layout::Position> {
        match self {
            RatzillaBackend::Dom(backend) => backend.get_cursor_position(),
            RatzillaBackend::Canvas(backend) => backend.get_cursor_position(),
            RatzillaBackend::WebGl2(backend) => backend.get_cursor_position(),
        }
    }

    fn set_cursor_position<P: Into<ratzilla::ratatui::layout::Position>>(
        &mut self,
        position: P,
    ) -> io::Result<()> {
        match self {
            RatzillaBackend::Dom(backend) => backend.set_cursor_position(position),
            RatzillaBackend::Canvas(backend) => backend.set_cursor_position(position),
            RatzillaBackend::WebGl2(backend) => backend.set_cursor_position(position),
        }
    }

    fn clear(&mut self) -> io::Result<()> {
        match self {
            RatzillaBackend::Dom(backend) => backend.clear(),
            RatzillaBackend::Canvas(backend) => backend.clear(),
            RatzillaBackend::WebGl2(backend) => backend.clear(),
        }
    }

    fn append_lines(&mut self, n: u16) -> io::Result<()> {
        match self {
            RatzillaBackend::Dom(backend) => backend.append_lines(n),
            RatzillaBackend::Canvas(backend) => backend.append_lines(n),
            RatzillaBackend::WebGl2(backend) => backend.append_lines(n),
        }
    }

    fn window_size(&mut self) -> io::Result<ratzilla::ratatui::backend::WindowSize> {
        match self {
            RatzillaBackend::Dom(backend) => backend.window_size(),
            RatzillaBackend::Canvas(backend) => backend.window_size(),
            RatzillaBackend::WebGl2(backend) => backend.window_size(),
        }
    }
}

impl Backend for FpsTrackingBackend {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a ratzilla::ratatui::buffer::Cell)>,
    {
        self.inner.draw(content)
    }

    fn flush(&mut self) -> io::Result<()> {
        let result = self.inner.flush();
        // Record frame after successful flush
        if result.is_ok() {
            record_frame();
        }
        result
    }

    fn size(&self) -> io::Result<ratzilla::ratatui::layout::Size> {
        self.inner.size()
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.inner.hide_cursor()
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.inner.show_cursor()
    }

    fn get_cursor_position(&mut self) -> io::Result<ratzilla::ratatui::layout::Position> {
        self.inner.get_cursor_position()
    }

    fn set_cursor_position<P: Into<ratzilla::ratatui::layout::Position>>(
        &mut self,
        position: P,
    ) -> io::Result<()> {
        self.inner.set_cursor_position(position)
    }

    fn clear(&mut self) -> io::Result<()> {
        self.inner.clear()
    }

    fn append_lines(&mut self, n: u16) -> io::Result<()> {
        self.inner.append_lines(n)
    }

    fn window_size(&mut self) -> io::Result<ratzilla::ratatui::backend::WindowSize> {
        self.inner.window_size()
    }
}

/// Read the backend type from query parameters, fallback to default
fn get_backend_from_query(default: BackendType) -> BackendType {
    let window = match web_sys::window() {
        Some(window) => window,
        None => return default,
    };

    let location = match window.location().href() {
        Ok(href) => href,
        Err(_) => return default,
    };

    let url = match Url::new(&location) {
        Ok(url) => url,
        Err(_) => return default,
    };

    let search_params = url.search_params();
    let backend_param = search_params.get("backend");

    match backend_param {
        Some(backend_str) => BackendType::from_str(&backend_str).unwrap_or(default),
        None => default,
    }
}

/// Inject HTML footer with backend switching links
fn inject_backend_footer(current_backend: BackendType) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    // Remove existing footer if present
    if let Some(existing) = document.get_element_by_id("ratzilla-backend-footer") {
        existing.remove();
    }

    // Create footer element
    let footer = document.create_element("div")?;
    footer.set_id("ratzilla-backend-footer");

    // Set footer styles
    footer.set_attribute(
        "style",
        "position: fixed; bottom: 0; left: 0; right: 0; \
         background: rgba(0,0,0,0.8); color: white; \
         padding: 8px 16px; font-family: monospace; font-size: 12px; \
         display: flex; justify-content: center; gap: 16px; \
         border-top: 1px solid #333; z-index: 1000;",
    )?;

    // Get current URL without backend param - use relative URL to avoid protocol issues
    let location = window.location();
    let base_url = location.pathname().unwrap_or_default();

    let backends = [BackendType::Dom, BackendType::Canvas, BackendType::WebGl2];
    let mut links = Vec::new();

    for backend in backends {
        let is_current = backend == current_backend;
        let style = if is_current {
            "color: #4ade80; font-weight: bold; text-decoration: none;"
        } else {
            "color: #94a3b8; text-decoration: none; cursor: pointer;"
        };

        let link = if is_current {
            format!(
                "<span style=\"{}\">● {}</span>",
                style,
                backend.display_name()
            )
        } else {
            format!(
                "<a href=\"{}?backend={}\" style=\"{}\">{}</a>",
                base_url,
                backend.as_str(),
                style,
                backend.display_name()
            )
        };

        links.push(link);
    }

    let footer_html = format!(
        "<span style=\"color: #64748b;\">Backend:</span> {} | \
         <span style=\"color: #64748b;\">FPS:</span> \
         <span id=\"ratzilla-fps\" style=\"color: #4ade80; font-weight: bold;\">--</span>",
        links.join(" | ")
    );

    footer.set_inner_html(&footer_html);

    // Append to body
    let body = document.body().ok_or("No body")?;
    body.append_child(&footer)?;

    Ok(())
}

fn create_backend_with_options(
    backend_type: BackendType,
    dom_options: Option<DomBackendOptions>,
    canvas_options: Option<CanvasBackendOptions>,
    webgl2_options: Option<WebGl2BackendOptions>,
) -> io::Result<RatzillaBackend> {
    use RatzillaBackend::*;
    match backend_type {
        BackendType::Dom => Ok(Dom(DomBackend::new_with_options(
            dom_options.unwrap_or_default(),
        )?)),
        BackendType::Canvas => Ok(Canvas(CanvasBackend::new_with_options(
            canvas_options.unwrap_or_default(),
        )?)),
        BackendType::WebGl2 => Ok(WebGl2(WebGl2Backend::new_with_options(
            webgl2_options.unwrap_or_default(),
        )?)),
    }
}
