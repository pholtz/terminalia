use crossterm::event::{KeyCode, KeyEvent};
use rltk::Point;
use specs::prelude::*;
use std::cmp::{max, min};

use crate::{
    App, RootScreen, RunState, Screen,
    component::{Attack, Item, Logbook, Player, Position, Stats, WantsToPickupItem},
    generate::map::{Map, TileType},
};

pub fn handle_main_explore_key_event(app: &mut App, runstate: RunState, key_event: KeyEvent) -> Option<RunState> {
    match key_event.code {
        KeyCode::Esc => {
            match runstate {
                RunState::Examining { index: _ } => Some(RunState::AwaitingInput),
                _ => {
                    app.root_screen = RootScreen::Menu;
                    None
                }
            }
        }

        KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('h') => {
            match runstate {
                RunState::AwaitingInput => try_move_player(-1, 0, &mut app.ecs),
                RunState::Examining { index: _ } => try_move_examine(app, -1, 0),
                _ => None,
            }
        }

        KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('l') => {
            match runstate {
                RunState::AwaitingInput => try_move_player(1, 0, &mut app.ecs),
                RunState::Examining { index: _ } => try_move_examine(app, 1, 0),
                _ => None,
            }
        }

        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
            match runstate {
                RunState::AwaitingInput => try_move_player(0, -1, &mut app.ecs),
                RunState::Examining { index: _ } => try_move_examine(app, 0, -1),
                _ => None,
            }
        }

        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
            match runstate {
                RunState::AwaitingInput => try_move_player(0, 1, &mut app.ecs),
                RunState::Examining { index: _ } => try_move_examine(app, 0, 1),
                _ => None,
            }
        }

        KeyCode::Char('/') => {
            let ecs = &mut app.ecs;
            let map = ecs.fetch::<Map>();
            let player = ecs.read_resource::<Entity>();
            let positions = ecs.read_storage::<Position>();
            let position = positions.get(*player).expect("Cannot get position for player");
            return match app.runstate {
                RunState::Examining { index: _ } => Some(RunState::AwaitingInput),
                _ => Some(RunState::Examining { index: map.xy_idx(position.x, position.y) })
            };
        }

        KeyCode::Char('g') => try_get_item(&mut app.ecs),
        KeyCode::Char('i') => {
            app.screen = Screen::Inventory;
            return None;
        }
        KeyCode::Char('.') => try_next_level(&mut app.ecs),
        KeyCode::Char('q') => {
            app.screen = Screen::Log;
            return None;
        }
        _ => None,
    }
}

fn try_move_examine(app: &mut App, delta_x: i32, delta_y: i32) -> Option<RunState> {
    match app.runstate {
        RunState::Examining { index } => {
            let map = app.ecs.fetch::<Map>();
            let (x, y) = map.idx_xy(index);
            return Some(RunState::Examining { index: map.xy_idx(x + delta_x, y + delta_y) });
        },
        _ => None
    }
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) -> Option<RunState> {
    let entities = ecs.entities();
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut attacks = ecs.write_storage::<Attack>();
    let stats = ecs.read_storage::<Stats>();
    let mut player_position = ecs.write_resource::<Point>();
    let map = ecs.fetch::<Map>();
    let mut _logbook = ecs.write_resource::<Logbook>();

    for (entity, pos, _player) in (&entities, &mut positions, &mut players).join() {
        let next_pos_x = min(map.width - 1, max(0, pos.x + delta_x));
        let next_pos_y = min(map.height - 1, max(0, pos.y + delta_y));
        let dest = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        for target in map.tile_content[dest].iter() {
            let target_stats = stats.get(*target);
            match target_stats {
                None => {}
                Some(_t) => {
                    attacks
                        .insert(entity, Attack { target: *target })
                        .expect("Unable to add attack");
                    return Some(RunState::PlayerTurn);
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
    return Some(RunState::PlayerTurn);
}

fn try_get_item(ecs: &mut World) -> Option<RunState> {
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
    return Some(RunState::PlayerTurn);
}

fn try_next_level(ecs: &mut World) -> Option<RunState> {
    let mut runstate = ecs.write_resource::<RunState>();
    let map = ecs.read_resource::<Map>();
    let player_position = ecs.read_resource::<Point>();
    let player_index = map.xy_idx(player_position.x, player_position.y);
    if map.tiles[player_index] == TileType::DownStairs {
        *runstate = RunState::Descending;
        return Some(RunState::Descending);
    }
    return None;
}
