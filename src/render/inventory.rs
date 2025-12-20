use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, palette::tailwind::SLATE},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph},
};
use specs::prelude::*;

use crate::{RunState, component::{Equipped, Inventory, Item, Name, Stats}, render::game::format_pools};

/**
 * This render function fires when the player is ingame and viewing their inventory.
 *
 * Render the players current inventory using the `Inventory` component on the player entity.
 * Includes the quantity per held item as well as any relevant equipment stats or descriptions.
 */
pub fn render_inventory(ecs: &mut World, runstate: RunState, frame: &mut Frame) {
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
        inventory_list.push(ListItem::new(Text::from(line)));
    }

    let mut state = ListState::default();
    if inventory_list.is_empty() {
        inventory_list.push(ListItem::from("Your inventory is empty!".to_string()));
    } else {
        state.select(Some(inventory.index));
    }

    let pools = format_pools(&player_entity, stats.clone(), inventories).expect("Unable to format player pools!");

    let fstat = format_stats(stat, runstate);

    let attribute_title = match runstate {
        RunState::LevelUp { index: _ } => "Level Up! Select an attribute to increase.",
        _ => "Attributes"
    };

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
            Line::from(Span::styled(
                name.name.clone(),
                Style::new().add_modifier(Modifier::ITALIC),
            )),
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
            Line::from(Span::styled(
                attribute_title,
                Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(fstat.strength.0, fstat.strength.1)),
            Line::from(Span::styled(fstat.dexterity.0, fstat.dexterity.1)),
            Line::from(Span::styled(fstat.constitution.0, fstat.constitution.1)),
            Line::from(Span::styled(fstat.intelligence.0, fstat.intelligence.1)),
            Line::from(Span::styled(fstat.wisdom.0, fstat.wisdom.1)),
            Line::from(Span::styled(fstat.charisma.0, fstat.charisma.1)),
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

pub struct FormattedStats {
    strength: (String, Style),
    dexterity: (String, Style),
    constitution: (String, Style),
    intelligence: (String, Style),
    wisdom: (String, Style),
    charisma: (String, Style),
}

fn format_stats(stat: &Stats, runstate: RunState) -> FormattedStats {
    let mut formatted = FormattedStats {
        strength: (format!("Strength: {}", stat.strength), Style::default()),
        dexterity: (format!("Dexterity: {}", stat.dexterity), Style::default()),
        constitution: (format!("Constitution: {}", stat.constitution), Style::default()),
        intelligence: (format!("Intelligence: {}", stat.intelligence), Style::default()),
        wisdom: (format!("Wisdom: {}", stat.wisdom), Style::default()),
        charisma: (format!("Charisma: {}", stat.charisma), Style::default()),
    };

    if let RunState::LevelUp { index } = runstate {
        match index {
            0 => formatted.strength = (
                format!("Strength: {} -> {}", stat.strength, stat.strength + 1),
                Style::new().fg(Color::DarkGray).bg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            1 => formatted.dexterity = (
                format!("Dexterity: {} -> {}", stat.dexterity, stat.dexterity + 1),
                Style::new().fg(Color::DarkGray).bg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            2 => formatted.constitution = (
                format!("Constitution: {} -> {}", stat.constitution, stat.constitution + 1),
                Style::new().fg(Color::DarkGray).bg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            3 => formatted.intelligence = (
                format!("Intelligence: {} -> {}", stat.intelligence, stat.intelligence + 1),
                Style::new().fg(Color::DarkGray).bg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            4 => formatted.wisdom = (
                format!("Wisdom: {} -> {}", stat.wisdom, stat.wisdom + 1),
                Style::new().fg(Color::DarkGray).bg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            5 => formatted.charisma = (
                format!("Charisma: {} -> {}", stat.charisma, stat.charisma + 1),
                Style::new().fg(Color::DarkGray).bg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            _ => {}
        }
    }
    return formatted;
}
