// This example is a stress test for the foreground rendering of the CanvasBackend.

mod fps;

use ratzilla::ratatui::layout::Size;
use ratzilla::ratatui::style::{Modifier, Style, Styled};
use ratzilla::ratatui::text::{Line, Span};
use ratzilla::ratatui::widgets::{Clear, Paragraph, Wrap};
use ratzilla::{
    ratatui::{layout::Rect, style::Color, widgets::Widget, Terminal},
    CanvasBackend, WebRenderer,
};
use std::cell::RefCell;
use std::rc::Rc;
use web_time::Instant;
use crate::fps::{FpsRecorder, FpsStats};

struct WidgetCache {
    white: Vec<Paragraph<'static>>,
    colorize_e_words: Vec<Paragraph<'static>>,
    colorize_some: Vec<Paragraph<'static>>,
    colorize_words: Vec<Paragraph<'static>>,
}

impl WidgetCache {
    const SCREEN_TYPES: usize = 4;
    const CACHED_SCREENS: usize = 10;

    fn new(area: Size) -> Self {
        fn white(_: &'static str, span: Span<'static>) -> Span<'static> {
            let style = span.style.fg(Color::White);
            span.set_style(style)
        }

        fn colorize_words(word: &'static str, span: Span<'static>) -> Span<'static> {
            let hash: usize = word.chars().map(|c| c as usize).sum();
            let color = COLORS[hash % COLORS.len()];
            let style = span.style.fg(color);
            span.set_style(style)
        }

        fn colorize_some(word: &'static str, span: Span<'static>) -> Span<'static> {
            let hash: usize = word.chars().take(1).map(|c| c as usize / 10).sum();
            let color = COLORS[hash % COLORS.len()];
            let style = span.style.fg(color);
            span.set_style(style)
        }

        fn colorize_e_words(word: &'static str, span: Span<'static>) -> Span<'static> {
            let c = if word.starts_with("e") {
                COLORS[7]
            } else {
                COLORS[0]
            };
            let style = span.style.fg(c);
            span.set_style(style)
        }

        let area = (area.width * area.height) as u32;
        let p = |f: fn(&'static str, Span<'static>) -> Span<'static>| {
            (0..10)
                .into_iter()
                .map(|i| lorem_ipsum_paragraph(area, i * Self::CACHED_SCREENS, f))
                .collect::<Vec<_>>()
        };

        Self {
            white: p(white),
            colorize_e_words: p(colorize_e_words),
            colorize_some: p(colorize_some),
            colorize_words: p(colorize_words),
        }
    }

    fn get(&self, style_type: usize, index: usize) -> &Paragraph<'static> {
        let index = index % Self::CACHED_SCREENS;
        match style_type {
            0 => &self.white[index],
            1 => &self.colorize_e_words[index],
            2 => &self.colorize_some[index],
            _ => &self.colorize_words[index],
        }
    }
}

fn main() -> std::io::Result<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let backend = CanvasBackend::new()?;
    let terminal = Terminal::new(backend)?;

    let mut fps_recorder = FpsRecorder::new();
    let mut rendered_frames = 0;

    let span_op_index = Rc::new(RefCell::new(0usize));

    let span_op_index_key_event = span_op_index.clone();
    terminal.on_key_event(move |event| {
        let cell = span_op_index_key_event.as_ref();
        let next_index = cell.borrow().clone() + 1;
        *cell.borrow_mut() = next_index % WidgetCache::SCREEN_TYPES;
    });

    let widget_cache = WidgetCache::new(terminal.size().unwrap());

    terminal.draw_web(move |frame| {
        let p = widget_cache.get(*span_op_index.as_ref().borrow(), rendered_frames);
        frame.render_widget(p, frame.area());
        rendered_frames += 1;
        fps_recorder.record();

        let fps_style = Style::default()
            .bg(Color::White)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD);
        
        FpsStats::new(&fps_recorder)
            .main_style(fps_style)
            .fps_value_style(fps_style)
            .render(frame.area(), frame.buffer_mut())
    });

    Ok(())
}

fn lorem_ipsum_paragraph(
    text_len: u32,
    word_offset: usize,
    span_op: impl Fn(&'static str, Span<'static>) -> Span<'static>,
) -> Paragraph<'static> {
    let spans = lorem_ipsum(text_len as _, word_offset).map(|w| span_op(w, Span::raw(w)));

    Paragraph::new(Line::from_iter(spans)).wrap(Wrap { trim: true })
}

fn lorem_ipsum(len: usize, word_offset: usize) -> impl Iterator<Item = &'static str> {
    let mut acc = 0;

    LOREM_IPSUM
        .iter()
        .copied()
        .cycle()
        .skip(word_offset * 2) // *2 to account for the space
        .flat_map(|w| [w, " "].into_iter())
        .take_while(move |w| {
            let is_within_screen = acc <= len;
            acc += w.len();
            is_within_screen
        })
}

const COLORS: [Color; 22] = [
    Color::from_u32(0xfbf1c7),
    Color::from_u32(0xfb4934),
    Color::from_u32(0xb8bb26),
    Color::from_u32(0xfabd2f),
    Color::from_u32(0x83a598),
    Color::from_u32(0xd3869b),
    Color::from_u32(0x8ec07c),
    Color::from_u32(0xfe8019),
    Color::from_u32(0xcc241d),
    Color::from_u32(0x98971a),
    Color::from_u32(0xd79921),
    Color::from_u32(0x458588),
    Color::from_u32(0xb16286),
    Color::from_u32(0x689d6a),
    Color::from_u32(0xd65d0e),
    Color::from_u32(0x9d0006),
    Color::from_u32(0x79740e),
    Color::from_u32(0xb57614),
    Color::from_u32(0x076678),
    Color::from_u32(0x8f3f71),
    Color::from_u32(0x427b58),
    Color::from_u32(0xaf3a03),
];

const LOREM_IPSUM: [&str; 69] = [
    "lorem",
    "ipsum",
    "dolor",
    "sit",
    "amet",
    "consectetur",
    "adipiscing",
    "elit",
    "sed",
    "do",
    "eiusmod",
    "tempor",
    "incididunt",
    "ut",
    "labore",
    "et",
    "dolore",
    "magna",
    "aliqua",
    "ut",
    "enim",
    "ad",
    "minim",
    "veniam",
    "quis",
    "nostrud",
    "exercitation",
    "ullamco",
    "laboris",
    "nisi",
    "ut",
    "aliquip",
    "ex",
    "ea",
    "commodo",
    "consequat",
    "duis",
    "aute",
    "irure",
    "dolor",
    "in",
    "reprehenderit",
    "in",
    "voluptate",
    "velit",
    "esse",
    "cillum",
    "dolore",
    "eu",
    "fugiat",
    "nulla",
    "pariatur",
    "excepteur",
    "sint",
    "occaecat",
    "cupidatat",
    "non",
    "proident",
    "sunt",
    "in",
    "culpa",
    "qui",
    "officia",
    "deserunt",
    "mollit",
    "anim",
    "id",
    "est",
    "laborum",
];
