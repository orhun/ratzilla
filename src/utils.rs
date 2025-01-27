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

/// Open a URL in a new tab or the current tab.
pub fn open_url(url: &str, new_tab: bool) -> Result<(), Error> {
    let window = web_sys::window().ok_or(Error::UnableToRetrieveWindow)?;
    if new_tab {
        window.open_with_url(url)?;
    } else {
        let location = window.location();
        location.set_href(url)?;
        location.replace(url)?;
    }
    Ok(())
}
