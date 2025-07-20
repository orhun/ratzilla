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

impl From<BackendType> for MultiBackendBuilder {
    fn from(backend_type: BackendType) -> Self {
        MultiBackendBuilder::with_fallback(backend_type)
    }
}

/// Enum wrapper for different Ratzilla backends that implements the Ratatui Backend trait.
/// 
/// This enum allows switching between different rendering backends at runtime while
/// providing a unified interface. All backend operations are delegated to the wrapped
/// backend implementation.
/// 
/// # Backends
/// 
/// - `Dom`: HTML DOM-based rendering with accessibility features
/// - `Canvas`: Canvas 2D API rendering with full Unicode support  
/// - `WebGl2`: GPU-accelerated rendering using WebGL2 and beamterm-renderer
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

/// Backend wrapper that automatically tracks FPS by recording frames on each flush.
///
/// This wrapper delegates all Backend trait methods to the inner RatzillaBackend
/// while recording frame timing information when `flush()` is called successfully.
/// The FPS data can be accessed through the `fps` module functions.
///
/// # Example
/// 
/// ```rust
/// let backend = RatzillaBackend::Dom(dom_backend);
/// let fps_backend = FpsTrackingBackend::new(backend);
/// let terminal = Terminal::new(fps_backend)?;
/// ```
pub struct FpsTrackingBackend {
    inner: RatzillaBackend,
}

impl FpsTrackingBackend {
    /// Create a new FPS tracking backend that wraps the given backend.
    /// 
    /// Frame timing will be recorded automatically on each successful flush operation.
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

/// Builder for creating terminals with different backend types and configuration options.
///
/// This builder provides a fluent API for configuring terminal and backend options
/// before creating a terminal instance. It supports automatic backend selection
/// from URL query parameters and includes FPS tracking by default.
///
/// # Backend Selection
///
/// The builder uses the following priority order for backend selection:
/// 1. `?backend=<type>` URL query parameter (dom, canvas, or webgl2)
/// 2. Fallback backend specified in `with_fallback()`
/// 3. Default backend (DOM)
///
/// # Example
///
/// ```rust
/// use ratzilla::backend::canvas::CanvasBackendOptions;
/// use ratzilla::ratatui::TerminalOptions;
/// 
/// let (backend_type, terminal) = MultiBackendBuilder::with_fallback(BackendType::Canvas)
///     .terminal_options(TerminalOptions::default())
///     .canvas_options(CanvasBackendOptions::default())
///     .build_terminal()?;
/// ```
#[derive(Debug, Default)]
pub struct MultiBackendBuilder {
    default_backend: BackendType,

    terminal_options: TerminalOptions,
    canvas_options: CanvasBackendOptions,
    dom_options: DomBackendOptions,
    webgl2_options: WebGl2BackendOptions,
}

impl MultiBackendBuilder {
    /// Create a new builder with the specified fallback backend type.
    ///
    /// The fallback backend will be used if no backend is specified in the URL query parameters.
    pub fn with_fallback(default_backend: BackendType) -> Self {
        Self {
            default_backend,
            ..Self::default()
        }
    }

    /// Set terminal configuration options.
    ///
    /// These options control terminal behavior such as viewport behavior and drawing settings.
    pub fn terminal_options(mut self, options: TerminalOptions) -> Self {
        self.terminal_options = options;
        self
    }

    /// Set options for the Canvas backend.
    ///
    /// These options control Canvas 2D rendering behavior such as font settings,
    /// cursor appearance, and Unicode support.
    pub fn canvas_options(mut self, options: CanvasBackendOptions) -> Self {
        self.canvas_options = options;
        self
    }

    /// Set options for the DOM backend.
    ///
    /// These options control DOM rendering behavior such as accessibility features,
    /// element styling, and focus management.
    pub fn dom_options(mut self, options: DomBackendOptions) -> Self {
        self.dom_options = options;
        self
    }

    /// Set options for the WebGL2 backend.
    ///
    /// These options control WebGL2 rendering behavior such as shader configuration,
    /// GPU memory management, and performance settings.
    pub fn webgl2_options(mut self, options: WebGl2BackendOptions) -> Self {
        self.webgl2_options = options;
        self
    }

    /// Build the terminal with the configured options and backend selection.
    ///
    /// This method:
    /// 1. Determines the backend type from URL query parameters or fallback
    /// 2. Creates the appropriate backend with the configured options
    /// 3. Wraps the backend with FPS tracking
    /// 4. Creates and returns the terminal with the selected backend
    /// 5. Injects a backend footer into the DOM (best effort)
    ///
    /// # Returns
    ///
    /// A tuple containing the selected backend type and the configured terminal instance.
    ///
    /// # Errors
    ///
    /// Returns an error if backend creation or terminal initialization fails.
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

/// Parse the backend type from URL query parameters, with fallback to default.
///
/// Checks for a `?backend=<type>` query parameter in the current page URL.
/// Valid backend types are "dom", "canvas", and "webgl2" (case-insensitive).
/// If no valid backend is found in the URL, returns the provided default.
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

/// Create a backend instance with the specified type and options.
///
/// Creates the appropriate backend variant (DOM, Canvas, or WebGL2) using the provided
/// configuration options. Options default to `Default::default()` if `None` is provided.
///
/// # Arguments
///
/// * `backend_type` - The type of backend to create
/// * `dom_options` - Configuration options for DOM backend (if applicable)
/// * `canvas_options` - Configuration options for Canvas backend (if applicable)  
/// * `webgl2_options` - Configuration options for WebGL2 backend (if applicable)
///
/// # Returns
///
/// The created backend wrapped in a `RatzillaBackend` enum.
///
/// # Errors
///
/// Returns an error if the backend creation fails (e.g., WebGL2 not supported).
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
