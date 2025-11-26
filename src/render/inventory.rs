use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style, palette::tailwind::SLATE},
    widgets::List,
};
use specs::prelude::*;

use crate::component::{InBackpack, Name};

pub fn render_inventory(ecs: &mut World, frame: &mut Frame) {
    let player_entity = ecs.fetch::<Entity>();
    let names = ecs.read_storage::<Name>();
    let backpack = ecs.read_storage::<InBackpack>();

    let inventory = (&names, &backpack)
        .join()
        .filter(|item| item.1.owner == *player_entity)
        .map(|item| item.0.name.clone())
        .collect::<Vec<_>>();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1)])
        .split(frame.area());

    frame.render_widget(
        List::new(inventory.into_iter())
            .highlight_style(Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD))
            .highlight_symbol(">")
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always),
        layout[0],
    );
}
