use crossterm::event::{KeyCode, KeyEvent};
use specs::prelude::*;

use crate::{component::{Inventory, WantsToConsumeItem}, App, Screen};

pub fn handle_main_inventory_key_event(app: &mut App, key_event: KeyEvent) -> bool {
    match key_event.code {
        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
            let player_entity = app.ecs.fetch::<Entity>();
            let mut inventories = app.ecs.write_storage::<Inventory>();
            if let Some(inventory) = inventories.get_mut(*player_entity) {
                if inventory.index > 0 {
                    inventory.index -= 1;
                }
            }
            return false;
        }

        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
            let player_entity = app.ecs.fetch::<Entity>();
            let mut inventories = app.ecs.write_storage::<Inventory>();
            if let Some(inventory) = inventories.get_mut(*player_entity) {
                if inventory.index + 1 < inventory.items.len() {
                    inventory.index += 1;
                }
            }
            return false;
        }

        KeyCode::Char('i') | KeyCode::Esc => {
            app.screen = Screen::Explore;
            return false;
        }
        KeyCode::Enter => {
            try_consume_item(&mut app.ecs);
            app.screen = Screen::Explore;
            return true;
        }
        _ => false,
    }
}

fn try_consume_item(ecs: &mut World) -> bool {
    let player_entity = ecs.fetch::<Entity>();
    let inventories = ecs.read_storage::<Inventory>();
    let mut wants_consume = ecs.write_storage::<WantsToConsumeItem>();

    if let Some(inventory) = inventories.get(*player_entity) {
        if let Some(item_stack) = inventory.items.get_index(inventory.index) {
            if let Some(item) = item_stack.1.get(0) {
                wants_consume
                    .insert(*player_entity, WantsToConsumeItem { item: *item })
                    .expect("Unable to insert item consumption into ecs");
            }
        }
    }
    return true;
}
