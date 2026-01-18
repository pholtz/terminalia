use ratatui::{Frame, layout::Alignment, style::{Color, Modifier, Style, palette::tailwind::SLATE}, text::{Line, Span, Text}, widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph}};
use specs::prelude::*;

use crate::{component::{Armor, Equipped, Item, MagicWeapon, MeleeWeapon, Name, RangedWeapon, Vendor}, render::inventory::format_inventory_item};

pub fn render_trading(ecs: &mut World, frame: &mut Frame, vendor_entity: Entity, index: usize) {
    let names = ecs.read_storage::<Name>();
    let items = ecs.read_storage::<Item>();
    let equipment = ecs.read_storage::<Equipped>();
    let melee_weapons = ecs.read_storage::<MeleeWeapon>();
    let ranged_weapons = ecs.read_storage::<RangedWeapon>();
    let magic_weapons = ecs.read_storage::<MagicWeapon>();
    let armors = ecs.read_storage::<Armor>();
    let vendors = ecs.read_storage::<Vendor>();
    
    let vendor = vendors.get(vendor_entity).expect("Unable to access given vendor component");

    let vendor_inventory: Vec<ListItem> = vendor.items
        .iter()
        .map(|item_entity| {
            let name = names.get(*item_entity).map(|name| name.name.clone()).unwrap_or("???".to_string());
            format_inventory_item(
                name,
                *item_entity,
                1,
                &items,
                &equipment,
                &melee_weapons,
                &ranged_weapons,
                &magic_weapons,
                &armors
            )
        })
        .collect();

    let mut state = ListState::default()
        .with_selected(Some(index));

    frame.render_stateful_widget(
        List::new(vendor_inventory)
            .block(
                Block::new()
                    .title("For sale")
                    .borders(Borders::ALL)
                    .title_alignment(Alignment::Center)
                    .padding(Padding::uniform(1)),
            )
            .highlight_style(Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD))
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Never),
        frame.area(),
        &mut state,
    );
}
