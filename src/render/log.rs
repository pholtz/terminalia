use std::{cmp, sync::atomic::Ordering};

use ratatui::{ Frame, layout::Alignment, widgets::{Block, Borders, Paragraph}};
use specs::prelude::*;

use crate::{logbook::logbook::{LOG_INDEX, format_text}, render::game::VIEW_HEIGHT};

/**
 * Renders the fullscreen logbook, when toggled.
 */
pub fn render_log(_ecs: &mut World, frame: &mut Frame) {
    let text = format_text(VIEW_HEIGHT as usize);
    let text_length = text.lines.len().try_into().expect("Unable to convert log length from usize -> u16");
    let index: u16 = LOG_INDEX.load(Ordering::Relaxed).try_into().expect("Unable to convert log_index from usize -> u16");
    let adjusted_index = cmp::min(text_length, index);
    frame.render_widget(
        Paragraph::new(format_text(VIEW_HEIGHT as usize))
            .scroll((adjusted_index, 0))
            .block(Block::bordered()
                .borders(Borders::ALL)
                .title("Logbook")
                .title_alignment(Alignment::Center)
            ),
        frame.area()
    );
}
