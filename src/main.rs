// warning: cobbled together code from the web_sys examples and chatgpt for help with requestAnimationFrame and closures in wasm
// here be dragons fo sho

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, Document, Element, HtmlElement};

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn create_span(document: &Document, text: &str, style: &str) -> Element {
    let span = document.create_element("span").unwrap();
    span.set_inner_html(text);
    span.set_attribute("style", style).unwrap();
    span
}

fn main() -> Result<(), JsValue> {
    // Access the document
    let window = window().unwrap();
    let document = window.document().unwrap();

    // Create the spans
    let span1 = create_span(&document, "Link", "color: red; font-weight: bold;");
    let span2 = create_span(&document, "foo", "color: green; cursor: pointer;");
    let span3 = create_span(&document, "ClickToRemove", "color: blue;");

    // Wrap the first span in an <a> element
    // NOTE: Ratatui has no Link widget atm as far as I know
    let anchor = document.create_element("a")?;
    anchor.set_attribute("href", "https://ratatui.rs")?;
    anchor.append_child(&span1)?;

    // Attach onclick event to the second span

    let span2_clone = span2.clone();
    let closure = Closure::wrap(Box::new(move || {
        // toggle the text of the span between "foo" and "bar"

        span2_clone.set_inner_html(if span2_clone.inner_html() == "foo" {
            "bar"
        } else {
            "foo"
        });
    }) as Box<dyn Fn()>);

    span2
        .dyn_ref::<HtmlElement>()
        .unwrap()
        .set_onclick(Some(closure.as_ref().unchecked_ref()));
    closure.forget(); // Prevent closure from being dropped

    // attach remove event to the last span
    let span3_clone = span3.clone();
    let closure = Closure::wrap(Box::new(move || span3_clone.remove()) as Box<dyn Fn()>);

    span3
        .dyn_ref::<HtmlElement>()
        .unwrap()
        .set_onclick(Some(closure.as_ref().unchecked_ref()));
    closure.forget(); // Prevent closure from being dropped

    // create div to hold the grid of characters
    let div = document.create_element("div").unwrap();
    div.set_attribute("id", "grid").unwrap();

    let x = 40;
    let y = 50;
    let mut grid: Vec<Element> = vec![];

    // create the grid
    for _i in 0..y {
        for _j in 0..x {
            let elem = create_span(&document, "A", "color: hsl(0, 100%, 50%);");
            div.append_child(&elem).unwrap();
            grid.push(elem);
        }
        div.append_child(&document.create_element("br").unwrap())
            .unwrap();
    }

    // Use requestAnimationFrame to change the color of the last span
    let span3_clone = span3.clone();
    let mut hue: i32 = 0;

    let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::new({
        let g = g.clone();
        move || {
            hue = (hue + 1) % 360;
            span3_clone
                .set_attribute("style", &format!("color: hsl({}, 100%, 50%);", hue))
                .unwrap();

            // update the grid
            for i in 0..y {
                for j in 0..x {
                    let elem = grid[i * x + j].clone();

                    elem.set_attribute(
                        "style",
                        &format!("color: hsl({}, 100%, 50%);", hue + ((i + j) as i32) % 360),
                    )
                    .unwrap();
                }
            }

            // Request the next animation frame - this is stupid, but I guess it works?
            window
                .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                .unwrap();
        }
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    // Add the elements to the document body
    let body = document.body().unwrap();
    body.append_child(&anchor)?;
    body.append_child(&span2)?;
    body.append_child(&span3)?;
    body.append_child(&div)?;

    Ok(())
}
