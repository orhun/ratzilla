use web_sys::{window, Document, Window};
use crate::error::Error;

pub(crate) fn get_document() -> Result<Document, Error> {
    get_window()?
        .document()
        .ok_or(Error::UnableToRetrieveDocument)
}

pub(crate) fn get_window() -> Result<Window, Error> {
    window().ok_or(Error::UnableToRetrieveWindow)
}

pub(crate) fn get_element_by_id_or_body(id: Option<&String>) -> Result<web_sys::Element, Error> {
    match id {
        Some(id) => get_document()?
            .get_element_by_id(id)
            .ok_or_else(|| Error::UnableToRetrieveElementById(id.to_string())),
        None => get_document()?
            .body()
            .ok_or(Error::UnableToRetrieveBody)
            .map(|body| body.into()),
    }
    
}