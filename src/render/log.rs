use std::{cmp, sync::atomic::Ordering};

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Position},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    App,
    logbook::logbook::{LOG_INDEX, format_text},
    render::game::VIEW_HEIGHT,
};

/**
 * Renders the fullscreen logbook, when toggled.
 * 
 * Also features a tabbable text input which allows players to write custom entries.
 */
pub fn render_log(app: &mut App, frame: &mut Frame) {
    let text = format_text(VIEW_HEIGHT as usize);
    let text_length = text
        .lines
        .len()
        .try_into()
        .expect("Unable to convert log length from usize -> u16");
    let index: u16 = LOG_INDEX
        .load(Ordering::Relaxed)
        .try_into()
        .expect("Unable to convert log_index from usize -> u16");
    let adjusted_index = cmp::min(text_length, index);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(1)]);
    let [log_area, input_area] = layout.areas(frame.area());

    frame.render_widget(
        Paragraph::new(format_text(VIEW_HEIGHT as usize))
            .wrap(Wrap { trim: true })
            .scroll((adjusted_index, 0))
            .block(
                Block::bordered()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(if app.log_index == 0 {
                        Color::Cyan
                    } else {
                        Color::default()
                    }))
                    .title("Logbook")
                    .title_alignment(Alignment::Center),
            ),
        log_area,
    );

    frame.render_widget(
        Paragraph::new(format!("> {}", app.logbook_input.clone())).style(Style::default().fg(
            if app.log_index == 1 {
                Color::Cyan
            } else {
                Color::default()
            },
        )),
        input_area,
    );

    if app.log_index == 1 {
        frame.set_cursor_position(Position {
            x: input_area.x + app.logbook_input.len() as u16 + 2,
            y: input_area.y,
        });
    }
}
