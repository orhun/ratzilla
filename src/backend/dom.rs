use std::{cell::RefCell, io::Result as IoResult, rc::Rc};

use ratatui::{
    backend::WindowSize,
    buffer::Cell,
    layout::{Position, Size},
    prelude::Backend,
};
use web_sys::{
    wasm_bindgen::{prelude::Closure, JsCast},
    window, Document, Element, Window,
};

use crate::{backend::utils::*, error::Error};

/// Options for the [`DomBackend`].
#[derive(Debug, Default)]
pub struct DomBackendOptions {
    /// The element ID.
    grid_id: Option<String>,
}

impl DomBackendOptions {
    /// Constructs a new [`DomBackendOptions`].
    pub fn new(grid_id: Option<String>) -> Self {
        Self { grid_id }
    }

    /// Returns the grid ID.
    ///
    /// - If the grid ID is not set, it returns `"grid"`.
    /// - If the grid ID is set, it returns the grid ID suffixed with
    ///   `"_ratzilla_grid"`.
    pub fn grid_id(&self) -> String {
        match &self.grid_id {
            Some(id) => format!("{id}_ratzilla_grid"),
            None => "grid".to_string(),
        }
    }
}

/// DOM backend.
///
/// This backend uses the DOM to render the content to the screen.
///
/// In other words, it transforms the [`Cell`]s into `<span>`s which are then
/// appended to a `<pre>` element.
#[derive(Debug)]
pub struct DomBackend {
    /// Whether the backend has been initialized.
    initialized: Rc<RefCell<bool>>,
    /// Current buffer.
    buffer: Vec<Vec<Cell>>,
    /// Previous buffer.
    prev_buffer: Vec<Vec<Cell>>,
    /// Cells.
    cells: Vec<Element>,
    /// Grid element.
    grid: Element,
    /// The parent of the grid element.
    grid_parent: Element,
    /// Window.
    window: Window,
    /// Document.
    document: Document,
    /// Options.
    options: DomBackendOptions,
}

impl DomBackend {
    /// Constructs a new [`DomBackend`].
    pub fn new() -> Result<Self, Error> {
        Self::new_with_options(DomBackendOptions::default())
    }

    /// Constructs a new [`DomBackend`] and uses the given element ID for the grid.
    pub fn new_by_id(id: &str) -> Result<Self, Error> {
        Self::new_with_options(DomBackendOptions::new(Some(id.to_string())))
    }

    /// Constructs a new [`DomBackend`] with the given options.
    pub fn new_with_options(options: DomBackendOptions) -> Result<Self, Error> {
        let window = window().ok_or(Error::UnableToRetrieveWindow)?;
        let document = window.document().ok_or(Error::UnableToRetrieveDocument)?;
        let mut backend = Self {
            initialized: Rc::new(RefCell::new(false)),
            buffer: vec![],
            prev_buffer: vec![],
            cells: vec![],
            grid: document.create_element("div")?,
            grid_parent: match options.grid_id.as_ref() {
                Some(id) => document
                    .get_element_by_id(id)
                    .ok_or(Error::UnableToRetrieveBody)?,
                None => document.body().ok_or(Error::UnableToRetrieveBody)?.into(),
            },
            options,
            window,
            document,
        };
        backend.add_on_resize_listener();
        backend.reset_grid()?;
        Ok(backend)
    }

    /// Add a listener to the window resize event.
    fn add_on_resize_listener(&mut self) {
        let initialized = self.initialized.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |_: web_sys::Event| {
            initialized.replace(false);
        });
        self.window
            .set_onresize(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    /// Reset the grid and clear the cells.
    fn reset_grid(&mut self) -> Result<(), Error> {
        self.grid = self.document.create_element("div")?;
        self.grid.set_attribute("id", &self.options.grid_id())?;
        self.cells.clear();
        self.buffer = get_sized_buffer();
        self.prev_buffer = self.buffer.clone();
        Ok(())
    }

    /// Pre-render the content to the screen.
    ///
    /// This function is called from [`flush`] once to render the initial
    /// content to the screen.
    fn prerender(&mut self) -> Result<(), Error> {
        for line in self.buffer.iter() {
            let mut line_cells: Vec<Element> = Vec::new();
            for cell in line.iter() {
                let span = create_span(&self.document, cell)?;
                self.cells.push(span.clone());
                line_cells.push(span);
            }

            // Create a <pre> element for the line
            let pre = self.document.create_element("pre")?;

            // Append all elements (spans and anchors) to the <pre>
            for elem in line_cells {
                pre.append_child(&elem)?;
            }

            // Append the <pre> to the grid
            self.grid.append_child(&pre)?;
        }
        Ok(())
    }

    /// Compare the current buffer to the previous buffer and updates the grid
    /// accordingly.
    fn update_grid(&mut self) -> Result<(), Error> {
        for (y, line) in self.buffer.iter().enumerate() {
            for (x, cell) in line.iter().enumerate() {
                if cell != &self.prev_buffer[y][x] {
                    let elem = self.cells[y * self.buffer[0].len() + x].clone();
                    elem.set_attribute("style", &get_cell_style_as_css(cell))?;
                    if let Some(anchor) = elem.first_element_child() {
                        if let Some(url) = cell.hyperlink() {
                            anchor.set_attribute("href", url)?;
                            anchor.set_inner_html(cell.symbol());
                        } else {
                            anchor.remove();
                            elem.set_inner_html(cell.symbol());
                        }
                    } else {
                        elem.set_inner_html(cell.symbol());
                    }
                }
            }
        }
        Ok(())
    }
}

impl Backend for DomBackend {
    // Populates the buffer with the given content.
    fn draw<'a, I>(&mut self, content: I) -> IoResult<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        if !*self.initialized.borrow() {
            // Only runs on resize event.
            if self
                .document
                .get_element_by_id(&self.options.grid_id())
                .is_some()
            {
                self.grid_parent.set_inner_html("");
                self.reset_grid()?;
            }
        }

        // Update the cells with new content
        for (x, y, cell) in content {
            let y = y as usize;
            let x = x as usize;
            if y < self.buffer.len() {
                let line = &mut self.buffer[y];
                line.extend(
                    std::iter::repeat_with(Cell::default).take(x.saturating_sub(line.len())),
                );
                if x < line.len() {
                    line[x] = cell.clone();
                }
            }
        }
        Ok(())
    }

    /// Flush the content to the screen.
    ///
    /// This function is called after the [`DomBackend::draw`] function to
    /// actually render the content to the screen.
    fn flush(&mut self) -> IoResult<()> {
        if !*self.initialized.borrow() {
            self.initialized.replace(true);
            self.grid_parent
                .append_child(&self.grid)
                .map_err(Error::from)?;
            self.prerender()?;
            // Set the previous buffer to the current buffer for the first render
            self.prev_buffer = self.buffer.clone();
        }
        // Check if the buffer has changed since the last render and update the grid
        if self.buffer != self.prev_buffer {
            self.update_grid()?;
        }
        self.prev_buffer = self.buffer.clone();
        Ok(())
    }

    fn hide_cursor(&mut self) -> IoResult<()> {
        Ok(())
    }

    fn show_cursor(&mut self) -> IoResult<()> {
        Ok(())
    }

    fn get_cursor(&mut self) -> IoResult<(u16, u16)> {
        Ok((0, 0))
    }

    fn set_cursor(&mut self, _x: u16, _y: u16) -> IoResult<()> {
        Ok(())
    }

    fn clear(&mut self) -> IoResult<()> {
        self.buffer = get_sized_buffer();
        Ok(())
    }

    fn size(&self) -> IoResult<Size> {
        Ok(Size::new(
            self.buffer[0].len().saturating_sub(1) as u16,
            self.buffer.len().saturating_sub(1) as u16,
        ))
    }

    fn window_size(&mut self) -> IoResult<WindowSize> {
        unimplemented!()
    }

    fn get_cursor_position(&mut self) -> IoResult<Position> {
        unimplemented!()
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, _: P) -> IoResult<()> {
        unimplemented!()
    }
}
