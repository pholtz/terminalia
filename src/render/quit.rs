use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Stylize},
    symbols::border,
    text::Text,
    widgets::{Block, Borders, Padding, Paragraph},
};
use specs::prelude::*;

pub fn render_quit(_ecs: &mut World, quit: bool, frame: &mut Frame) {
    let menu = Block::default()
        .borders(Borders::all())
        .padding(Padding::symmetric(5, 6))
        .inner(frame.area());

    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .split(menu);

    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Fill(1),
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Fill(1),
        ])
        .split(vertical_layout[2]);

    frame.render_widget(
        Paragraph::new(Text::from("Would you like to quit?")).centered(),
        vertical_layout[1],
    );

    frame.render_widget(
        Paragraph::new(Text::from("No"))
            .centered()
            .bg(if quit { Color::Black } else { Color::Cyan })
            .block(Block::bordered().border_set(border::THICK)),
        horizontal_layout[1],
    );

    frame.render_widget(
        Paragraph::new(Text::from("Yes"))
            .centered()
            .bg(if quit { Color::Cyan } else { Color::Black })
            .block(Block::bordered().border_set(border::THICK)),
        horizontal_layout[3],
    );
}
