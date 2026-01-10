use std::{cmp, sync::atomic::Ordering};

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Position},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};

use crate::{
    App,
    logbook::logbook::{LOG_INDEX, format_all_text},
};

/**
 * Renders the fullscreen logbook, when toggled.
 * 
 * Also features a tabbable text input which allows players to write custom entries.
 */
pub fn render_log(app: &mut App, frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(1)
        ]);
    let [log_area, input_area] = layout.areas(frame.area());

    let log_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(log_area);

    let index: u16 = LOG_INDEX
        .load(Ordering::Relaxed)
        .try_into()
        .expect("Unable to convert log_index from usize -> u16");
    // let text = format_text(index as usize, (log_area.height - 2) as usize);
    let text = format_all_text();
    let text_length = text
        .lines
        .len()
        .try_into()
        .expect("Unable to convert log length from usize -> u16");
    let adjusted_index = cmp::min(text_length, index);

    let mut scrollbar_state = ScrollbarState::default()
        .content_length(text_length as usize)
        .position(adjusted_index as usize);

    frame.render_widget(
        Paragraph::new(text)
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
        log_layout[0],
    );
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        log_layout[1],
        &mut scrollbar_state,
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
