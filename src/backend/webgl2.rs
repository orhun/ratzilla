use crate::backend::elements::get_element_by_id_or_body;
use crate::{backend::utils::*, error::Error, CursorShape};
use beamterm_renderer::{CellData, FontAtlas, FontStyle, GlyphEffect, Renderer, TerminalGrid};
use ratatui::{
    backend::WindowSize,
    buffer::Cell,
    layout::{Position, Size},
    prelude::Backend,
    style::{Color, Modifier},
};
use std::io::Result as IoResult;
use std::mem::swap;

/// Options for the [`CanvasBackend`].
#[derive(Debug, Default)]
pub struct WebGl2BackendOptions {
    /// The element ID.
    grid_id: Option<String>,
    /// Override the automatically detected size.
    size: Option<(u32, u32)>,
    /// Measure performance using the `performance` API.
    measure_performance: bool,
}

impl WebGl2BackendOptions {
    /// Constructs a new [`CanvasBackendOptions`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the element id of the canvas' parent element.
    pub fn grid_id(mut self, id: &str) -> Self {
        self.grid_id = Some(id.to_string());
        self
    }

    /// Sets the size of the canvas, in pixels.
    pub fn size(mut self, size: (u32, u32)) -> Self {
        self.size = Some(size);
        self
    }
}

/// WebGl2 renderer and context.
#[derive(Debug)]
struct WebGl2 {
    /// The WebGL2 renderer.
    renderer: Renderer,
    /// Drawable representation of the terminal
    terminal_grid: TerminalGrid,
    /// Background color.
    background_color: Color,
}

impl WebGl2 {
    /// Constructs a new [`Canvas`].
    fn new(
        parent_element: web_sys::Element,
        width: u32,
        height: u32,
        background_color: Color,
    ) -> Result<Self, Error> {
        let canvas = create_canvas_in_element(&parent_element, width, height)?;

        let renderer = Renderer::create_with_canvas(canvas)?;
        let terminal_grid = TerminalGrid::new(
            renderer.gl(),
            FontAtlas::load_default(renderer.gl())?,
            renderer.canvas_size(),
        )?;

        Ok(Self {
            terminal_grid,
            renderer,
            background_color,
        })
    }
    
    fn terminal_size(&self) -> (u16, u16) {
        self.terminal_grid.terminal_size()
    }
}

/// WebGl2 backend.
///
/// This backend renders the buffer onto a HTML canvas element.
#[derive(Debug)]
pub struct WebGl2Backend {
    /// Current buffer.
    buffer: Vec<Cell>,
    /// Indicates if the cells have changed, requiring a redraw.
    cell_data_pending_upload: bool,
    /// WebGl2 context.
    context: WebGl2,
    /// Cursor position.
    cursor_position: Option<Position>,
    /// The cursor shape.
    cursor_shape: CursorShape,
    /// Performance measurement.
    performance: Option<web_sys::Performance>,
}

impl WebGl2Backend {
    /// Constructs a new [`CanvasBackend`].
    pub fn new() -> Result<Self, Error> {
        let (width, height) = get_raw_window_size();
        Self::new_with_size(width.into(), height.into())
    }

    /// Constructs a new [`CanvasBackend`] with the given size.
    pub fn new_with_size(width: u32, height: u32) -> Result<Self, Error> {
        Self::new_with_options(WebGl2BackendOptions {
            size: Some((width, height)),
            ..Default::default()
        })
    }

    /// Constructs a new [`CanvasBackend`] with the given options.
    pub fn new_with_options(options: WebGl2BackendOptions) -> Result<Self, Error> {
        let performance = if options.measure_performance {
            Some(performance()?)
        } else {
            Some(performance()?)
            // None
        };

        // Parent element of canvas (uses <body> unless specified)
        let parent = get_element_by_id_or_body(options.grid_id.as_ref())?;

        let (width, height) = options
            .size
            .unwrap_or_else(|| (parent.client_width() as u32, parent.client_height() as u32));

        let context = WebGl2::new(parent, width, height, Color::Black)?;
        let buffer = get_sized_buffer_from_terminal_grid(&context.terminal_grid);
        Ok(Self {
            buffer,
            cell_data_pending_upload: false,
            context,
            cursor_position: None,
            cursor_shape: CursorShape::SteadyBlock,
            performance,
        })
    }

    /// Sets the background color of the canvas.
    pub fn set_background_color(&mut self, color: Color) {
        // todo: propagte to renderer
        self.context.background_color = color;
    }

    /// Returns the [`CursorShape`].
    pub fn cursor_shape(&self) -> &CursorShape {
        &self.cursor_shape
    }

    /// Set the [`CursorShape`].
    pub fn set_cursor_shape(mut self, shape: CursorShape) -> Self {
        self.cursor_shape = shape;
        self
    }

    // Compare the current buffer to the previous buffer and updates the canvas
    // accordingly.
    fn update_grid(&mut self) -> Result<(), Error> {
        self.measure_begin("update-grid");
        if self.cell_data_pending_upload {
            let gl = self.context.renderer.gl();
            let terminal = &mut self.context.terminal_grid;
            let cells = self.buffer.iter().map(cell_data);
            
            terminal.update_cells(gl, cells)?;
            
            self.cell_data_pending_upload = false;
        }
        self.measure_end("update-grid");

        Ok(())
    }

    /// Draws the cursor at the specified position. 
    fn draw_cursor(&mut self, pos: Position) -> IoResult<()> {
        let w = self.context.terminal_size().0 as usize;
        let idx = pos.y as usize * w + pos.x as usize;
        
        if idx < self.buffer.len() {
            let cursor_style = self.cursor_shape.show(self.buffer[idx].style());
            self.buffer[idx].set_style(cursor_style);
        }
        
        Ok(())
    }

    /// Measures the beginning of a performance mark.
    fn measure_begin(&self, label: &str) {
        if let Some(performance) = &self.performance {
            performance.mark(label)
                .unwrap_or_default();
        }
    }

    /// Measures the end of a performance mark.
    fn measure_end(&self, label: &str) {
        if let Some(performance) = &self.performance {
            performance.measure_with_start_mark(label, label)
                .unwrap_or_default();
        }
    }
}

/// Converts a [`term_renderer::Error`] into a [`Error`].
impl From<beamterm_renderer::Error> for Error {
    fn from(value: beamterm_renderer::Error) -> Self {
        use beamterm_renderer::Error::*;
        match value {
            Initialization(s) 
            | Shader(s) 
            | Resource(s) 
            | Data(s) => Error::WebGl2Error(s),
        }
    }
}

impl Backend for WebGl2Backend {
    // Populates the buffer with the given content.
    fn draw<'a, I>(&mut self, content: I) -> IoResult<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        let w = self.context.terminal_grid.terminal_size().0 as usize;

        self.measure_begin("draw-grid");
        let terminal = &mut self.context.terminal_grid;
        self.context.renderer.render(terminal);
        self.measure_end("draw-grid");

        self.measure_begin("update-cell-content");
        let mut sync_required = false;
        for (x, y, received_cell) in content {
            let (x, y) = (x as usize, y as usize);

            let current_cell = &mut self.buffer[y * w + x];
            let new_cell = cell_with_safe_colors(received_cell);
            sync_required |= current_cell != &new_cell;
            *current_cell = new_cell;
        }
        self.cell_data_pending_upload = sync_required;
        self.measure_end("update-cell-content");
        
        // Draw the cursor if set
        if let Some(pos) = self.cursor_position {
            self.draw_cursor(pos)?;
        }

        Ok(())
    }

    
    /// Flush the content to the screen.
    ///
    /// This function is called after the [`CanvasBackend::draw`] function to
    /// actually render the content to the screen.
    fn flush(&mut self) -> IoResult<()> {
        self.update_grid()?;
        Ok(())
    }

    fn hide_cursor(&mut self) -> IoResult<()> {
        if let Some(pos) = self.cursor_position {
            let y = pos.y as usize;
            let x = pos.x as usize;
            let w = self.context.terminal_grid.terminal_size().0 as usize;

            if let Some(cell) = self.buffer.get_mut(y * w + x) {
                let style = self.cursor_shape.hide(cell.style());
                cell.set_style(style);
            }
        }

        self.cursor_position = None;
        Ok(())
    }

    fn show_cursor(&mut self) -> IoResult<()> {
        Ok(())
    }

    fn get_cursor(&mut self) -> IoResult<(u16, u16)> {
        if let Some(pos) = self.cursor_position {
            Ok((pos.x, pos.y))
        } else {
            Ok((0, 0)) // Default position if cursor is not set
        }
    }

    fn set_cursor(&mut self, x: u16, y: u16) -> IoResult<()> {
        self.cursor_position = Some(Position::new(x, y));
        Ok(())
    }

    fn clear(&mut self) -> IoResult<()> {
        // todo: clear canvas
        Ok(())
    }

    fn size(&self) -> IoResult<Size> {
        let (w, h) = self.context.terminal_grid.terminal_size();
        Ok(Size::new(w as _, h as _))
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
        let w = self.context.terminal_grid.terminal_size().0 as usize;

        let new_pos = position.into();
        if let Some(old_pos) = self.cursor_position {
            if old_pos == new_pos {
                return Ok(()); // No change in cursor position
            }

            let y = old_pos.y as usize;
            let x = old_pos.x as usize;

            let old_idx = y * w + x;
            if let Some(old_cell) = self.buffer.get_mut(old_idx) {
                let style = self.cursor_shape.hide(old_cell.style());
                old_cell.set_style(style);
            }
        }
        self.cursor_position = Some(new_pos);
        Ok(())
    }
}

/// Returns a buffer based on the `TerminalGrid`.
fn get_sized_buffer_from_terminal_grid(grid: &TerminalGrid) -> Vec<Cell> {
    vec![Cell::default(); grid.cell_count()]
}

fn to_rgb(color: Color) -> u32 {
    let c = match color {
        Color::Rgb(r, g, b) => ((r as u32) << 16) | ((g as u32) << 8) | b as u32,
        Color::Reset => 0x000000,
        Color::Black => 0x000000,
        Color::Red => 0x800000,
        Color::Green => 0x008000,
        Color::Yellow => 0x808000,
        Color::Blue => 0x000080,
        Color::Magenta => 0x800080,
        Color::Cyan => 0x008080,
        Color::Gray => 0xc0c0c0,
        Color::DarkGray => 0x808080,
        Color::LightRed => 0xFF0000,
        Color::LightGreen => 0x00FF00,
        Color::LightYellow => 0xFFFF00,
        Color::LightBlue => 0x0000FF,
        Color::LightMagenta => 0xFF00FF,
        Color::LightCyan => 0x00FFFF,
        Color::White => 0xFFFFFF,
        Color::Indexed(code) => indexed_to_rgb(code)
    };

    c
}

fn cell_with_safe_colors(cell: &Cell) -> Cell {
    let mut fg = if cell.fg == Color::Reset {
        Color::White
    } else {
        cell.fg
    };

    let mut bg = if cell.bg == Color::Reset {
        Color::Black
    } else {
        cell.bg
    };

    if cell.modifier.contains(Modifier::REVERSED) {
        swap(&mut fg, &mut bg);
    }
    
    let mut c = cell.clone();
    c.set_fg(fg);
    c.set_bg(bg);
    c
}

fn indexed_to_rgb(index: u8) -> u32 {
    match index {
        // Basic 16 colors (0-15)
        0..=15 => {
            const BASIC_COLORS: [u32; 16] = [
                0x000000, // 0: black
                0xCD0000, // 1: red
                0x00CD00, // 2: green
                0xCDCD00, // 3: yellow
                0x0000EE, // 4: blue
                0xCD00CD, // 5: magenta
                0x00CDCD, // 6: cyan
                0xE5E5E5, // 7: white
                0x7F7F7F, // 8: bright Black
                0xFF0000, // 9: bright Red
                0x00FF00, // 10: bright Green
                0xFFFF00, // 11: bright Yellow
                0x5C5CFF, // 12: bright Blue
                0xFF00FF, // 13: bright Magenta
                0x00FFFF, // 14: bright Cyan
                0xFFFFFF, // 15: bright White
            ];
            BASIC_COLORS[index as usize]
        }

        // 216-color cube (16-231)
        16..=231 => {
            let cube_index = index - 16;
            let r = cube_index / 36;
            let g = (cube_index % 36) / 6;
            let b = cube_index % 6;

            // Convert 0-5 range to 0-255 RGB
            // Values: 0 -> 0, 1 -> 95, 2 -> 135, 3 -> 175, 4 -> 215, 5 -> 255
            let to_rgb = |n: u8| -> u32 {
                if n == 0 { 0 } else { 55 + 40 * n as u32 }
            };

            to_rgb(r) << 16 | to_rgb(g) << 8 | to_rgb(b)
        }

        // 24 grayscale colors (232-255)
        232..=255 => {
            let gray_index = index - 232;
            // linear interpolation from 8 to 238
            let gray = (8 + gray_index * 10) as u32;
            (gray << 16) | (gray << 8) | gray
        }
    }
}


fn cell_data(cell: &Cell) -> CellData {
    CellData::new_with_style_bits(
        cell.symbol(),
        into_glyph_bits(cell.modifier),
        to_rgb(cell.fg),
        to_rgb(cell.bg)
    )
}

/// Extracts glyph styling bits from cell modifiers.
///
/// # Performance Optimization
/// Bitwise operations are instead of individual `contains()` checks,
/// This provides a ~50% performance improvement over the naive approach.
///
/// # Bit Layout Reference
///
/// ```plain
/// Modifier bits:     0000_0000_0000_0001  (BOLD at bit 0)
///                    0000_0000_0000_0100  (ITALIC at bit 2)
///                    0000_0000_0000_1000  (UNDERLINED at bit 3)
///                    0000_0001_0000_0000  (CROSSED_OUT at bit 8)
///
/// FontStyle bits:    0000_0010_0000_0000  (Bold as bit 9)
///                    0000_0100_0000_0000  (Italic as bit 10)
/// GlyphEffect bits:  0001_0000_0000_0000  (Underline at bit 12)
///                    0010_0000_0000_0000  (Strikethrough at bit 13)
///
/// Shift operations:  bit 0 << 9 = bit 9
///                    bit 2 << 8 = bit 10
///                    bit 3 << 9 = bit 12
///                    bit 8 << 5 = bit 13
/// ```
const fn into_glyph_bits(modifier: Modifier) -> u16 {
    let m = modifier.bits();

    (m << 9) & (1 << 9)    // bold
    | (m << 8) & (1 << 10) // italic
    | (m << 9) & (1 << 12) // underline
    | (m << 5) & (1 << 13) // strikethrough
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Modifier;

    #[test]
    fn test_font_style() {
        [
            (FontStyle::Bold,       Modifier::BOLD),
            (FontStyle::Italic,     Modifier::ITALIC),
            (FontStyle::BoldItalic, Modifier::BOLD | Modifier::ITALIC),
        ].into_iter()
            .map(|(style, modifier)| (style as u16, into_glyph_bits(modifier)))
            .for_each(|(expected, actual)| assert_eq!(expected, actual));
    }

    #[test]
    fn test_glyph_effect() {
        [
            (GlyphEffect::Underline,     Modifier::UNDERLINED),
            (GlyphEffect::Strikethrough, Modifier::CROSSED_OUT),
        ].into_iter()
            .map(|(effect, modifier)| (effect as u16, into_glyph_bits(modifier)))
            .for_each(|(expected, actual)| assert_eq!(expected, actual));
    }
}