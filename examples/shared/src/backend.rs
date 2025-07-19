use std::io;
use web_sys::{window, Url};
use ratzilla::backend::canvas::CanvasBackendOptions;
use ratzilla::backend::dom::DomBackendOptions;
use ratzilla::backend::webgl2::WebGl2BackendOptions;
use ratzilla::{CanvasBackend, DomBackend, WebGl2Backend};
use ratzilla::ratatui::{Terminal, TerminalOptions};
use ratzilla::ratatui::backend::Backend;
use crate::fps;
use crate::utils::inject_backend_footer;


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

/// Backend wrapper that automatically tracks FPS
pub struct FpsTrackingBackend {
    inner: RatzillaBackend,
}

impl FpsTrackingBackend {
    pub fn new(backend: RatzillaBackend) -> Self {
        Self { inner: backend }
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
            fps::record_frame();
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
        let backend_type = parse_backend_from_url(self.default_backend);
        let backend = create_backend_with_options(
            backend_type,
            Some(self.dom_options),
            Some(self.canvas_options),
            Some(self.webgl2_options),
        )?;

        // Initialize FPS recorder
        fps::init_fps_recorder();

        // Wrap backend with FPS tracking
        let fps_backend = FpsTrackingBackend::new(backend);
        let terminal = Terminal::with_options(fps_backend, self.terminal_options)?;

        // Inject footer (ignore errors)
        let _ = inject_backend_footer(backend_type);

        Ok((backend_type, terminal))
    }
}

/// Create a new MultiBackendBuilder with the default backend type
pub fn multi_backend_builder(default: BackendType) -> MultiBackendBuilder {
    MultiBackendBuilder::new(default)
}

/// Read the backend type from query parameters, fallback to default
fn parse_backend_from_url(default: BackendType) -> BackendType {
    let backend_param = window()
        .map(|w| w.location())
        .and_then(|l| l.href().ok())
        .and_then(|url| Url::new(url.as_str()).ok())
        .and_then(|url| url.search_params().get("backend"));

    match backend_param {
        Some(backend_str) => BackendType::from_str(&backend_str).unwrap_or(default),
        None => default,
    }
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
