use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, palette::tailwind::SLATE},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph},
};
use specs::prelude::*;

use crate::{component::{Equipped, Inventory, Item, Name, Stats}, render::game::format_pools};

/**
 * This render function fires when the player is ingame and viewing their inventory.
 *
 * Render the players current inventory using the `Inventory` component on the player entity.
 * Includes the quantity per held item as well as any relevant equipment stats or descriptions.
 */
pub fn render_inventory(ecs: &mut World, frame: &mut Frame) {
    let player_entity = ecs.fetch::<Entity>();
    let inventories = ecs.read_storage::<Inventory>();
    let items = ecs.read_storage::<Item>();
    let equipment = ecs.read_storage::<Equipped>();
    let names = ecs.read_storage::<Name>();
    let stats = ecs.read_storage::<Stats>();

    let inventory = inventories
        .get(*player_entity)
        .expect("Unable to retrieve the player's inventory!");

    let stat = stats
        .get(*player_entity)
        .expect("Unable to retrieve the player's stats!");

    let name = names
        .get(*player_entity)
        .expect("Unable to retrieve the player's name!");

    /*
     * Create a formatted list of each inventory item.
     * For the selected item, render the description as well (if present).
     */
    let mut inventory_list = Vec::new();
    for (index, (key, value)) in inventory.items.iter().enumerate() {
        let item = value
            .first()
            .expect("Unable to retrieve inventory item entity");
        let equip_label = if equipment.contains(*item) {
            "(equipped)"
        } else {
            ""
        };

        let mut line = vec![
            "".into(),
            format!("{} x{} {}", key, value.len(), equip_label).into(),
            "".into(),
        ];
        if index == inventory.index {
            let description = value
                .first()
                .map(|item| {
                    items
                        .get(*item)
                        .expect("Unable to retrieve item component for existing inventory item")
                        .description
                        .clone()
                })
                .unwrap_or_else(|| "???".to_string());
            line.push(description.into());
            line.push("".into());
        }
        inventory_list.push(ListItem::new(line));
    }

    let mut state = ListState::default();
    if inventory_list.is_empty() {
        inventory_list.push(ListItem::from("Your inventory is empty!".to_string()));
    } else {
        state.select(Some(inventory.index));
    }

    let pools = format_pools(&player_entity, stats.clone(), inventories).expect("Unable to format player pools!");

    let root_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(frame.area());

    let inventory_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1)])
        .split(root_layout[0]);

    let character_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1)])
        .split(root_layout[1]);

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
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Never),
        inventory_layout[0],
        &mut state,
    );

    frame.render_widget(
        Paragraph::new(Text::from(vec![
            Line::from(name.name.clone()),
            Line::from(""),
            Line::from(vec![
                Span::styled(format!("{:12}", pools.hp.1), Style::new().fg(Color::LightRed)),
                Span::styled(pools.hp.2, Style::new().bg(Color::Red)),
                Span::styled(pools.hp.3, Style::new().bg(Color::Rgb(60, 0, 0))),
            ]),
            Line::from(vec![
                Span::styled(format!("{:12}", pools.mp.1), Style::new().fg(Color::Blue)),
                Span::styled(pools.mp.2, Style::new().bg(Color::Blue)),
                Span::styled(pools.mp.3, Style::new().bg(Color::Rgb(0, 0, 60))),
            ]),
            Line::from(vec![
                Span::styled(format!("{:12}", pools.exp.1), Style::new().fg(Color::LightMagenta)),
                Span::styled(pools.exp.2, Style::new().bg(Color::LightMagenta)),
                Span::styled(pools.exp.3, Style::new().bg(Color::Rgb(60, 60, 60))),
            ]),
            Line::from(""),
            Line::from(format!("Strength: {}", stat.strength)),
            Line::from(format!("Dexterity: {}", stat.dexterity)),
            Line::from(format!("Constitution: {}", stat.constitution)),
            Line::from(format!("Intelligence: {}", stat.intelligence)),
            Line::from(format!("Wisdom: {}", stat.wisdom)),
            Line::from(format!("Charisma: {}", stat.charisma)),
        ])).block(
            Block::new()
                .title("Character")
                .borders(Borders::ALL)
                .title_alignment(Alignment::Center)
                .padding(Padding::uniform(1)),
        ),
        character_layout[0],
    );
}
