use ratatui::{text::Text, widgets::Paragraph, Frame};
use specs::prelude::*;

use crate::{component::Logbook, generate::map::MAX_HEIGHT};

/**
 * Renders the fullscreen logbook, when toggled.
 */
pub fn render_log(ecs: &mut World, frame: &mut Frame) {
    let logbook = ecs.fetch::<Logbook>();
    let recent_entries = logbook.entries.len().saturating_sub(MAX_HEIGHT as usize);
    let mut serialized_log = String::with_capacity(1024);
    for entry in &logbook.entries[recent_entries..] {
        serialized_log.push_str(entry);
        serialized_log.push('\n');
    }
    frame.render_widget(
        Paragraph::new(Text::raw(serialized_log)).scroll((logbook.scroll_offset, 0)),
        frame.area(),
    );
}
