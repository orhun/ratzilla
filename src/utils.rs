use crate::error::Error;

/// Sets the document title.
pub fn set_document_title(title: &str) -> Result<(), Error> {
    web_sys::window()
        .ok_or(Error::UnableToRetrieveWindow)?
        .document()
        .ok_or(Error::UnableToRetrieveDocument)?
        .set_title(title);

    Ok(())
}
