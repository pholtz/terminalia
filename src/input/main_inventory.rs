use crossterm::event::{KeyCode, KeyEvent};
use specs::prelude::*;

use crate::{
    App, RunState, Screen,
    component::{Inventory, Stats, WantsToConsumeItem},
};

pub fn handle_main_inventory_key_event(app: &mut App, key_event: KeyEvent) -> Option<RunState> {
    if let RunState::LevelUp { index } = app.runstate {
        return handle_main_level_up_key_event(app, index, key_event);
    }

    match key_event.code {
        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
            let player_entity = app.ecs.fetch::<Entity>();
            let mut inventories = app.ecs.write_storage::<Inventory>();
            if let Some(inventory) = inventories.get_mut(*player_entity) {
                if inventory.index > 0 {
                    inventory.index -= 1;
                }
            }
            return None;
        }

        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
            let player_entity = app.ecs.fetch::<Entity>();
            let mut inventories = app.ecs.write_storage::<Inventory>();
            if let Some(inventory) = inventories.get_mut(*player_entity) {
                if inventory.index + 1 < inventory.items.len() {
                    inventory.index += 1;
                }
            }
            return None;
        }

        KeyCode::Char('i') | KeyCode::Esc => {
            app.screen = Screen::Explore;
            return None;
        }

        // Consume without leaving inventory screen
        KeyCode::Char(' ') => {
            try_consume_item(&mut app.ecs);
            return None;
        }

        // Consume and return to explore screen
        KeyCode::Enter => {
            try_consume_item(&mut app.ecs);
            app.screen = Screen::Explore;
            return None;
        }
        _ => None,
    }
}

fn handle_main_level_up_key_event(app: &mut App, index: usize, key_event: KeyEvent) -> Option<RunState> {
    match key_event.code {
        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
            if index > 0 {
                Some(RunState::LevelUp { index: index - 1 }) 
            } else {
                None
            }
        }
        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
            if index < 5 {
                Some(RunState::LevelUp { index: index + 1 })
            } else {
                None
            }
        }
        KeyCode::Enter => {
            let mut stats = app.ecs.write_storage::<Stats>();
            let player = app.ecs.fetch::<Entity>();
            if let Some(stat) = stats.get_mut(*player) {
                match index {
                    0 => stat.strength += 1,
                    1 => stat.dexterity += 1,
                    2 => stat.constitution += 1,
                    3 => stat.intelligence += 1,
                    4 => stat.wisdom += 1,
                    5 => stat.charisma += 1,
                    _ => panic!("ahhh"),
                }
            }
            Some(RunState::AwaitingInput)
        }
        _ => None,
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
