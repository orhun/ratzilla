use ratatui::layout::Size;

use crate::{
    backend::utils::{get_raw_screen_size, get_raw_window_size},
    error::Error,
};

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

/// Returns `true` if the screen is a mobile device.
pub fn is_mobile() -> bool {
    // TODO: Improve this...
    get_raw_screen_size().0 < 550
}

/// Returns the number of characters that can fit in the window (viewport of the browser or terminal).
pub fn get_window_size() -> Size {
    let (w, h) = get_raw_window_size();
    // TODO: These are mildly magical numbers... make them more precise
    (w / 10, h / 20).into()
}

/// Returns the number of characters that can fit in the screen (entire physical display).
pub fn get_screen_size() -> Size {
    let (w, h) = get_raw_screen_size();
    // TODO: These are mildly magical numbers... make them more precise
    (w as u16 / 10, h as u16 / 19).into()
}
