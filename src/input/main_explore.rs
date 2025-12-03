use crossterm::event::{KeyCode, KeyEvent};
use rltk::Point;
use specs::prelude::*;
use std::cmp::{max, min};

use crate::{
    App, RootScreen, RunState, Screen,
    component::{Attack, Item, Logbook, Player, Position, Stats, WantsToPickupItem},
    generate::map::{MAX_HEIGHT, MAX_WIDTH, Map, TileType, xy_idx},
};

pub fn handle_main_explore_key_event(app: &mut App, key_event: KeyEvent) -> bool {
    match key_event.code {
        KeyCode::Esc => {
            app.root_screen = RootScreen::Menu;
            return false;
        }

        KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('h') => {
            try_move_player(-1, 0, &mut app.ecs)
        }

        KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('l') => {
            try_move_player(1, 0, &mut app.ecs)
        }

        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
            try_move_player(0, -1, &mut app.ecs)
        }

        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
            try_move_player(0, 1, &mut app.ecs)
        }

        KeyCode::Char('g') => try_get_item(&mut app.ecs),
        KeyCode::Char('i') => {
            app.screen = Screen::Inventory;
            return false;
        }
        KeyCode::Char('.') => try_next_level(&mut app.ecs),
        KeyCode::Char('q') => {
            app.screen = Screen::Log;
            return false;
        }
        _ => false,
    }
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) -> bool {
    let entities = ecs.entities();
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut attacks = ecs.write_storage::<Attack>();
    let stats = ecs.read_storage::<Stats>();
    let mut player_position = ecs.write_resource::<Point>();
    let map = ecs.fetch::<Map>();
    let mut _logbook = ecs.write_resource::<Logbook>();

    for (entity, pos, _player) in (&entities, &mut positions, &mut players).join() {
        let next_pos_x = min(MAX_WIDTH - 1, max(0, pos.x + delta_x));
        let next_pos_y = min(MAX_HEIGHT - 1, max(0, pos.y + delta_y));
        let dest = xy_idx(pos.x + delta_x, pos.y + delta_y);

        for target in map.tile_content[dest].iter() {
            let target_stats = stats.get(*target);
            match target_stats {
                None => {}
                Some(_t) => {
                    attacks
                        .insert(entity, Attack { target: *target })
                        .expect("Unable to add attack");
                    return true;
                }
            }
        }

        let is_blocked_tile = map.blocked_tiles[dest];
        if !is_blocked_tile {
            pos.x = next_pos_x;
            pos.y = next_pos_y;
            player_position.x = next_pos_x;
            player_position.y = next_pos_y;
        }
    }
    return true;
}

fn try_get_item(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut logbook = ecs.fetch_mut::<Logbook>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => logbook
            .entries
            .push("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(
                    *player_entity,
                    WantsToPickupItem {
                        collected_by: *player_entity,
                        item: item,
                    },
                )
                .expect("Unable to insert item pickup into ecs");
        }
    }
    return true;
}

fn try_next_level(ecs: &mut World) -> bool {
    let mut runstate = ecs.write_resource::<RunState>();
    let map = ecs.read_resource::<Map>();
    let player_position = ecs.read_resource::<Point>();
    let player_index = xy_idx(player_position.x, player_position.y);
    if map.tiles[player_index] == TileType::DownStairs {
        *runstate = RunState::Descending;
        return true;
    }
    return false;
}
