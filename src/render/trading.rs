use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style, palette::tailwind::SLATE},
    widgets::{Block, Borders, List, ListItem, ListState, Padding},
};
use specs::prelude::*;

use crate::{
    component::{
        Armor, Equipped, Inventory, Item, MagicWeapon, MeleeWeapon, Name, RangedWeapon, Vendor,
    },
    logbook::logbook::format_latest_text,
    render::{inventory::format_inventory_item},
};

pub fn render_trading(
    ecs: &mut World,
    frame: &mut Frame,
    vendor_entity: Entity,
    vendor_index: usize,
    player_index: usize,
    is_buying: bool,
) {
    let player_entity = ecs.fetch::<Entity>();
    let names = ecs.read_storage::<Name>();
    let items = ecs.read_storage::<Item>();
    let equipment = ecs.read_storage::<Equipped>();
    let inventories = ecs.read_storage::<Inventory>();
    let melee_weapons = ecs.read_storage::<MeleeWeapon>();
    let ranged_weapons = ecs.read_storage::<RangedWeapon>();
    let magic_weapons = ecs.read_storage::<MagicWeapon>();
    let armors = ecs.read_storage::<Armor>();
    let vendors = ecs.read_storage::<Vendor>();

    let inventory = inventories
        .get(*player_entity)
        .expect("Unable to retrieve the player's inventory!");

    let vendor = vendors
        .get(vendor_entity)
        .expect("Unable to access given vendor component");

    let vendor_inventory_list: Vec<ListItem> = vendor
        .items
        .iter()
        .map(|item_entity| {
            let name = names
                .get(*item_entity)
                .map(|name| name.name.clone())
                .unwrap_or("???".to_string());
            format_inventory_item(
                name,
                *item_entity,
                1,
                &items,
                &equipment,
                &melee_weapons,
                &ranged_weapons,
                &magic_weapons,
                &armors,
            )
        })
        .collect();

    let player_inventory_list: Vec<ListItem> = inventory
        .items
        .iter()
        .enumerate()
        .map(|item| {
            format_inventory_item(
                item.1.0.clone(),
                item.1
                    .1
                    .first()
                    .expect("Unable to retrieve inventory item entity (top of stack)")
                    .clone(),
                item.1.1.len(),
                &items,
                &equipment,
                &melee_weapons,
                &ranged_weapons,
                &magic_weapons,
                &armors,
            )
        })
        .collect();

    let [trading_area, log_area] = Layout::new(
        Direction::Vertical,
        vec![Constraint::Percentage(80), Constraint::Percentage(20)],
    )
    .areas(frame.area());

    let [vendor_inventory_area, player_inventory_area] = Layout::new(
        Direction::Horizontal,
        vec![Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .areas(trading_area);

    frame.render_stateful_widget(
        List::new(vendor_inventory_list)
            .block(
                Block::new()
                    .title("For sale")
                    .borders(Borders::ALL)
                    .title_alignment(Alignment::Center)
                    .padding(Padding::uniform(1)),
            )
            .highlight_style(if is_buying {
                Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            })
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Never),
        vendor_inventory_area,
        &mut ListState::default().with_selected(Some(vendor_index)),
    );

    frame.render_stateful_widget(
        List::new(player_inventory_list)
            .block(
                Block::new()
                    .title(format!("My inventory ({} gold)", inventory.gold))
                    .borders(Borders::ALL)
                    .title_alignment(Alignment::Center)
                    .padding(Padding::uniform(1)),
            )
            .highlight_style(if !is_buying {
                Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            })
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Never),
        player_inventory_area,
        &mut ListState::default().with_selected(Some(player_index)),
    );

    frame.render_widget(format_latest_text(log_area.height as usize), log_area);
}
