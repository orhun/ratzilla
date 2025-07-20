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

use crate::{
    backend::utils::{MouseEventHandler, *},
    error::Error,
    event::MouseEvent,
    widgets::hyperlink::HYPERLINK_MODIFIER,
    CursorShape,
};

/// Options for the [`DomBackend`].
#[derive(Default, Debug)]
pub struct DomBackendOptions {
    /// The element ID.
    grid_id: Option<String>,
    /// The cursor shape.
    cursor_shape: CursorShape,
    /// Mouse event handler for the canvas.
    mouse_event_handler: Option<MouseEventHandler>,
}

impl DomBackendOptions {
    /// Constructs a new [`DomBackendOptions`].
    pub fn new(grid_id: Option<String>, cursor_shape: CursorShape) -> Self {
        Self {
            grid_id,
            cursor_shape,
            mouse_event_handler: None,
        }
    }

    /// Returns the grid ID.
    ///
    /// - If the grid ID is not set, it returns `"grid"`.
    /// - If the grid ID is set, it returns the grid ID suffixed with
    ///     `"_ratzilla_grid"`.
    pub fn grid_id(&self) -> String {
        match &self.grid_id {
            Some(id) => format!("{id}_ratzilla_grid"),
            None => "grid".to_string(),
        }
    }

    /// Returns the [`CursorShape`].
    pub fn cursor_shape(&self) -> &CursorShape {
        &self.cursor_shape
    }

    /// Configures a mouse event handler for the canvas.
    pub fn mouse_event_handler<F>(mut self, handler: F) -> Self
    where
        F: FnMut(MouseEvent) + 'static,
    {
        self.mouse_event_handler = Some(MouseEventHandler::new(handler));
        self
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
    /// Cursor position.
    cursor_position: Option<Position>,
    /// Cached cell size in pixels (width, height).
    cell_size_px: Option<(u32, u32)>,
    /// Mouse event handler if configured.
    mouse_event_handler: Option<MouseEventHandler>,
}

impl DomBackend {
    /// Constructs a new [`DomBackend`].
    pub fn new() -> Result<Self, Error> {
        Self::new_with_options(DomBackendOptions::default())
    }

    /// Constructs a new [`DomBackend`] and uses the given element ID for the grid.
    pub fn new_by_id(id: &str) -> Result<Self, Error> {
        Self::new_with_options(DomBackendOptions::new(
            Some(id.to_string()),
            CursorShape::default(),
        ))
    }

    /// Set the [`CursorShape`].
    pub fn set_cursor_shape(mut self, shape: CursorShape) -> Self {
        self.options.cursor_shape = shape;
        self
    }

    /// Constructs a new [`DomBackend`] with the given options.
    pub fn new_with_options(mut options: DomBackendOptions) -> Result<Self, Error> {
        let window = window().ok_or(Error::UnableToRetrieveWindow)?;
        let document = window.document().ok_or(Error::UnableToRetrieveDocument)?;

        let mouse_event_handler = options.mouse_event_handler.take();

        let mut backend = Self {
            initialized: Rc::new(RefCell::new(false)),
            buffer: vec![],
            prev_buffer: vec![],
            cells: vec![],
            grid: document.create_element("div")?,
            grid_parent: get_element_by_id_or_body(options.grid_id.as_ref())?,
            options,
            window,
            document,
            cursor_position: None,
            cell_size_px: None,
            mouse_event_handler,
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
        // Reset cell size cache when grid is reset
        self.cell_size_px = None;
        Ok(())
    }

    /// Measures and caches the actual cell size in pixels.
    fn measure_cell_size(&mut self) -> Result<(u32, u32), Error> {
        if let Some(cached_size) = self.cell_size_px {
            return Ok(cached_size);
        }

        let cell_size = measure_dom_cell_size(&self.document, &self.grid_parent)?;
        self.cell_size_px = Some(cell_size);
        Ok(cell_size)
    }

    /// Sets up mouse event handling for the DOM grid.
    fn setup_mouse_event_handler(&mut self) -> Result<(), Error> {
        if self.mouse_event_handler.is_none() {
            return Ok(());
        }

        let cell_size_px = self.measure_cell_size()?;
        let mut handler = self.mouse_event_handler.take().unwrap();

        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            // Get the grid element's bounding rectangle for proper coordinate calculation
            if let Some(target) = event.current_target() {
                if let Ok(element) = target.dyn_into::<web_sys::Element>() {
                    let rect = element.get_bounding_client_rect();
                    let grid_rect = (rect.left(), rect.top(), rect.width(), rect.height());
                    handler.call(MouseEvent::new_relative(event, cell_size_px, grid_rect));
                } else {
                    // Fallback to viewport-relative coordinates if we can't get the element
                    handler.call(MouseEvent::new(event, cell_size_px));
                }
            } else {
                // Fallback to viewport-relative coordinates if no target
                handler.call(MouseEvent::new(event, cell_size_px));
            }
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);

        self.grid
            .add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        self.grid
            .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        self.grid
            .add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;

        closure.forget();
        Ok(())
    }

    /// Pre-render the content to the screen.
    ///
    /// This function is called from [`flush`] once to render the initial
    /// content to the screen.
    fn prerender(&mut self) -> Result<(), Error> {
        for line in self.buffer.iter() {
            let mut line_cells: Vec<Element> = Vec::new();
            let mut hyperlink: Vec<Cell> = Vec::new();
            for (i, cell) in line.iter().enumerate() {
                if cell.modifier.contains(HYPERLINK_MODIFIER) {
                    hyperlink.push(cell.clone());
                    // If the next cell is not part of the hyperlink, close it
                    if !line
                        .get(i + 1)
                        .map(|c| c.modifier.contains(HYPERLINK_MODIFIER))
                        .unwrap_or(false)
                    {
                        let anchor = create_anchor(&self.document, &hyperlink)?;
                        for link_cell in &hyperlink {
                            let span = create_span(&self.document, link_cell)?;
                            self.cells.push(span.clone());
                            anchor.append_child(&span)?;
                        }
                        line_cells.push(anchor);
                        hyperlink.clear();
                    }
                } else {
                    let span = create_span(&self.document, cell)?;
                    self.cells.push(span.clone());
                    line_cells.push(span);
                }
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
                if cell.modifier.contains(HYPERLINK_MODIFIER) {
                    continue;
                }
                if cell != &self.prev_buffer[y][x] {
                    let elem = self.cells[y * self.buffer[0].len() + x].clone();
                    elem.set_inner_html(cell.symbol());
                    elem.set_attribute("style", &get_cell_style_as_css(cell))?;
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

        // Draw the cursor if set
        if let Some(pos) = self.cursor_position {
            let y = pos.y as usize;
            let x = pos.x as usize;
            let line = &mut self.buffer[y];
            if x < line.len() {
                let cursor_style = self.options.cursor_shape().show(line[x].style());
                line[x].set_style(cursor_style);
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

            // Set up mouse event handler after the grid is rendered
            self.setup_mouse_event_handler()?;
        }
        // Check if the buffer has changed since the last render and update the grid
        if self.buffer != self.prev_buffer {
            self.update_grid()?;
        }
        self.prev_buffer = self.buffer.clone();
        Ok(())
    }

    fn hide_cursor(&mut self) -> IoResult<()> {
        if let Some(pos) = self.cursor_position {
            let y = pos.y as usize;
            let x = pos.x as usize;
            let line = &mut self.buffer[y];
            if x < line.len() {
                let style = self.options.cursor_shape.hide(line[x].style());
                line[x].set_style(style);
            }
        }
        self.cursor_position = None;
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
        match self.cursor_position {
            None => Ok((0, 0).into()),
            Some(position) => Ok(position),
        }
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> IoResult<()> {
        let new_pos = position.into();
        if let Some(old_pos) = self.cursor_position {
            let y = old_pos.y as usize;
            let x = old_pos.x as usize;
            let line = &mut self.buffer[y];
            if x < line.len() && old_pos != new_pos {
                let style = self.options.cursor_shape.hide(line[x].style());
                line[x].set_style(style);
            }
        }
        self.cursor_position = Some(new_pos);
        Ok(())
    }
}

impl std::fmt::Debug for DomBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomBackend")
            .field("initialized", &self.initialized)
            .field(
                "buffer",
                &format!(
                    "{}x{} buffer",
                    self.buffer.len(),
                    self.buffer.first().map_or(0, |row| row.len())
                ),
            )
            .field(
                "prev_buffer",
                &format!(
                    "{}x{} buffer",
                    self.prev_buffer.len(),
                    self.prev_buffer.first().map_or(0, |row| row.len())
                ),
            )
            .field("cells", &format!("{} cells", self.cells.len()))
            .field("options", &self.options)
            .field("cursor_position", &self.cursor_position)
            .field("cell_size_px", &self.cell_size_px)
            .field("mouse_event_handler", &self.mouse_event_handler.is_some())
            .finish()
    }
}
