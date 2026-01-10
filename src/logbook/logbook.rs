use std::sync::{Mutex, atomic::{AtomicU16}};

use lazy_static::lazy_static;
use ratatui::{style::{Color, Style}, text::{Line, Span, Text}};

pub struct Logger {
    current_color: Color,
    fragments: Vec<LogFragment>,
}

impl Logger {
    pub fn new() -> Self {
        Logger {
            current_color: Color::White,
            fragments: Vec::new(),
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.current_color = color;
        self
    }

    pub fn append<T: ToString>(mut self, text: T) -> Self {
        self.fragments.push(LogFragment {
            color: self.current_color,
            text: text.to_string(),
        });
        self
    }

    pub fn append_with_color<T: ToString>(mut self, color: Color, text: T) -> Self {
        self.fragments.push(LogFragment {
            color: color,
            text: text.to_string(),
        });
        self
    }

    pub fn log(self) {
        append_many(self.fragments);
    }
}

pub struct LogFragment {
    pub color: Color,
    pub text: String,
}

/*****************************
 * Static log data structure *
 *****************************/
lazy_static! {
    static ref LOGBOOK: Mutex<Vec<Vec<LogFragment>>> = Mutex::new(Vec::new());
}
pub static LOG_INDEX: AtomicU16 = AtomicU16::new(0);

pub fn append_many(fragments: Vec<LogFragment>) {
    LOGBOOK.lock().unwrap().push(fragments);
}

/// Iterate over all elements in the log...
/// then format them into `Line` instances inside of a `Text` instance, to be
/// rendered by ratatui.
pub fn format_all_text() -> Text<'static> {
    let mut lines: Vec<Line> = Vec::new();
    for line in LOGBOOK.lock().unwrap().iter() {
        let mut spans: Vec<Span> = Vec::new();
        for chunk in line.iter() {
            spans.push(Span::styled(chunk.text.clone(), Style::new().fg(chunk.color)));
        }
        lines.push(Line::from(spans));
    }
    return Text::from(lines);
}

/// Strip the given `limit` number of entries off of the end of the logbook,
/// then format them into `Line` instances inside of a `Text` instance, to be
/// rendered by ratatui.
pub fn format_latest_text(limit: usize) -> Text<'static> {
    let mut lines: Vec<Line> = Vec::new();
    for line in LOGBOOK.lock().unwrap().iter().rev().take(limit).rev() {
        let mut spans: Vec<Span> = Vec::new();
        for chunk in line.iter() {
            spans.push(Span::styled(chunk.text.clone(), Style::new().fg(chunk.color)));
        }
        lines.push(Line::from(spans));
    }
    return Text::from(lines);
}

pub fn size() -> usize {
    LOGBOOK.lock().unwrap().len()
}

pub fn clear() {
    LOGBOOK.lock().unwrap().clear();
}
