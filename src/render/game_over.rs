use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Text},
    widgets::Paragraph,
};

pub fn render_game_over(frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(1),
            Constraint::Percentage(40),
        ])
        .split(frame.area());

    frame.render_widget(
        Paragraph::new(Text::from(Span::styled(
            "Y O U  D I E D",
            Style::default().fg(Color::Red),
        )))
        .centered(),
        layout[1],
    );
}
