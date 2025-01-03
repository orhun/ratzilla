use web_sys::{Document, Element};

pub fn create_span(document: &Document, text: &str, style: &str) -> Element {
    let span = document.create_element("span").unwrap();
    span.set_inner_html(text);
    span.set_attribute("style", style).unwrap();

    let pre = document.create_element("pre").unwrap();
    pre.set_attribute("style", "margin: 0px;").unwrap();
    pre.append_child(&span).unwrap();

    pre
}
