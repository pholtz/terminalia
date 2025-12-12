use std::sync::Mutex;

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

    pub fn log(self) {
        append_many(self.fragments);
    }
}

pub struct LogFragment {
    pub color: Color,
    pub text: String,
}

lazy_static! {
    static ref LOG: Mutex<Vec<Vec<LogFragment>>> = Mutex::new(Vec::new());
}

pub fn append(fragment: LogFragment) {
    LOG.lock().unwrap().push(vec![fragment]);
}

pub fn append_many(fragments: Vec<LogFragment>) {
    LOG.lock().unwrap().push(fragments);
}

pub fn format_text(limit: usize) -> Text<'static> {
    let mut lines: Vec<Line> = Vec::new();
    for line in LOG.lock().unwrap().iter().rev().take(limit) {
        let mut spans: Vec<Span> = Vec::new();
        for chunk in line.iter() {
            spans.push(Span::styled(chunk.text.clone(), Style::new().fg(chunk.color)));
        }
        lines.push(Line::from(spans));
    }
    return Text::from(lines);
}

pub fn clear() {
    LOG.lock().unwrap().clear();
}
