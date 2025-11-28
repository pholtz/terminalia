use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout}, style::{palette::tailwind::SLATE, Modifier, Style}, widgets::{Block, Borders, List, ListItem, ListState, Padding}, Frame
};
use specs::prelude::*;

use crate::component::Inventory;

pub fn render_inventory(ecs: &mut World, frame: &mut Frame) {
    let player_entity = ecs.fetch::<Entity>();
    let inventories = ecs.read_storage::<Inventory>();

    let inventory = inventories
        .get(*player_entity)
        .expect("Unable to retrieve the player's inventory!");

    let mut inventory_list = Vec::new();
    for (key, value) in &inventory.items {
        inventory_list.push(ListItem::new(vec![
            format!("{} x{}", key, value.len()).into(),
            "".into(),
        ]));
    }

    if inventory_list.is_empty() {
        inventory_list.push(ListItem::from("Your inventory is empty!".to_string()));
    }

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1)])
        .split(frame.area());

    let mut state = ListState::default();
    state.select_first();

    frame.render_stateful_widget(
        List::new(inventory_list)
            .block(
                Block::new()
                    .title("Inventory")
                    .borders(Borders::ALL)
                    .title_alignment(Alignment::Center)
                    .padding(Padding::uniform(1)),
            )
            .highlight_style(Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD))
            .highlight_symbol(">")
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always),
        layout[0],
        &mut state,
    );
}
