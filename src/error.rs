use web_sys::wasm_bindgen;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to retrieve window")]
    UnableToRetrieveWindow,

    #[error("Unable to retrieve document")]
    UnableToRetrieveDocument,

    #[error("Unable to retrieve body")]
    UnableToRetrieveBody,

    #[error("Unable to retrieve canvas context")]
    UnableToRetrieveCanvasContext,

    #[error("JS value error: {0:?}")]
    JsValue(wasm_bindgen::JsValue),
}

impl From<wasm_bindgen::JsValue> for Error {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        Self::JsValue(value)
    }
}

impl From<Error> for std::io::Error {
    fn from(error: Error) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, error.to_string())
    }
}
