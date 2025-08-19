use web_sys::wasm_bindgen;

/// Custom error implementation.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Unable to retrieve window.
    ///
    /// This error occurs when [`web_sys::window()`] returns `None`.
    #[error("Unable to retrieve window")]
    UnableToRetrieveWindow,

    /// Unable to retrieve document.
    ///
    /// This error occurs when `window.document()` returns `None`.
    #[error("Unable to retrieve document")]
    UnableToRetrieveDocument,

    /// Unable to retrieve body.
    ///
    /// This error occurs when `document.body()` returns `None`.
    #[error("Unable to retrieve body")]
    UnableToRetrieveBody,

    /// Unable to retrieve HTML element by ID.
    #[error("Unable to retrieve element: {0}")]
    UnableToRetrieveElementById(String),

    /// Unable to retrieve canvas context.
    ///
    /// This error occurs when `canvas.get_context_with_context_options("2d")`
    /// returns `None`.
    #[error("Unable to retrieve canvas context")]
    UnableToRetrieveCanvasContext,

    /// JS value error.
    #[error("JS value error: {0:?}")]
    JsValue(wasm_bindgen::JsValue),

    /// WebGL2Backend error.
    #[error("WebGL2Backend error: {0}")]
    WebGl2Error(beamterm_renderer::Error),

    /// Failed to retrieve a HTML/js component, such as `Performance`.
    #[error("Failed to retrieve component: {0}")]
    UnableToRetrieveComponent(&'static str),

    /// Mouse events are not supported by this backend.
    #[error("Mouse events are not supported by this backend")]
    MouseEventsNotSupported,
}

/// Convert [`wasm_bindgen::JsValue`] to [`Error`].
impl From<wasm_bindgen::JsValue> for Error {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        Self::JsValue(value)
    }
}

/// Convert [`Error`] to [`std::io::Error`].
impl From<Error> for std::io::Error {
    fn from(error: Error) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, error.to_string())
    }
}

/// Converts a [`beamterm_renderer::Error`] into a [`Error`].
impl From<beamterm_renderer::Error> for Error {
    fn from(value: beamterm_renderer::Error) -> Self {
        Self::WebGl2Error(value)
    }
}
