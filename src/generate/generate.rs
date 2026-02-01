use std::ops::Deref;
use rand::{Rng};

use crate::{
    App, Player, Position, RunState, component::{InBackpack, OtherLevelPosition}, generate::{
        map::{Map, MapOptions}, spawn::{spawn_npc_captain, spawn_npc_merchant, spawn_player, spawn_weighted_item, spawn_weighted_monster}
    }
};
use rltk::{Point, RandomNumberGenerator};
use specs::prelude::*;

/// Performs all of the associated mutations for switching a floor.
/// 
/// Functionality is encapsulated here to reduce complexoity in the main run fn.
/// Since ascending and descending are functionally quite similar, we handle
/// them both here.
pub fn switch_floor(app: &mut App, is_descending: bool) -> RunState {
    let next_index = if is_descending { app.floor_index + 1 } else { app.floor_index - 1};
    let existing_map = app.dungeon.get_map(next_index);
    freeze_floor(app.floor_index, &mut app.ecs);
    let mut map = match existing_map {
        Some(map) => {
            thaw_floor(next_index, &mut app.ecs);
            map
        }
        None => {
            let new_map = generate_floor(rand::rng().random(), next_index as u32, &mut app.ecs);
            app.dungeon.add_map(&new_map);                                    
            new_map
        }
    };

    // Maybe these should actually just be part of thawing the floor?
    let (player_x, player_y) = map.idx_xy(map.player_spawn_index.expect("No player spawn index"));
    map.clear_tile_content();
    app.ecs.insert(map);
    app.ecs.insert(RunState::AwaitingInput);
    app.ecs.insert(Point::new(player_x, player_y));
    app.floor_index = next_index;

    let player_entity = app.ecs.fetch::<Entity>();
    let mut positions = app.ecs.write_storage::<Position>();
    if let Some(position) = positions.get_mut(*player_entity) {
        position.x = player_x;
        position.y = player_y;
    }

    return RunState::AwaitingInput;
}

/// Given a mutable world, iterate over all entities and remove any that
/// should not carry over to the next floor.
///
/// Resources are not removed because we expect them to be preserved (logbook),
/// or overwritten (map) when setting up the next floor.
pub fn reset_floor(world: &mut World) {
    let mut to_delete: Vec<Entity> = Vec::new();
    {
        let entities = world.entities();
        let players = world.read_storage::<Player>();
        let backpacks = world.read_storage::<InBackpack>();
        let player_entity = world.fetch::<Entity>();

        for entity in entities.join() {
            if let Some(_player) = players.get(entity) {
                continue;
            }
            if let Some(backpack) = backpacks.get(entity) {
                if backpack.owner == *player_entity {
                    continue;
                }
            }
            to_delete.push(entity);
        }
    }
    let _ = world.delete_entities(&to_delete);
}

pub fn freeze_floor(index: u32, world: &mut World) {
    let entities = world.entities();
    let mut positions = world.write_storage::<Position>();
    let mut other_level_positions = world.write_storage::<OtherLevelPosition>();
    let player_entity = world.fetch::<Entity>();

    let mut to_delete: Vec<Entity> = Vec::new();
    for (entity, position) in (&entities, &positions).join() {
        if entity == *player_entity { continue; }
        other_level_positions.insert(entity, OtherLevelPosition {
            index: index,
            x: position.x,
            y: position.y,
        }).expect("Unable to insert OtherLevelPosition during freeze");
        to_delete.push(entity);
    }

    for delete in to_delete.iter() {
        positions.remove(*delete);
    }
}

pub fn thaw_floor(index: u32, world: &mut World) {
    let entities = world.entities();
    let mut positions = world.write_storage::<Position>();
    let mut other_level_positions = world.write_storage::<OtherLevelPosition>();
    let player_entity = world.fetch::<Entity>();

    let mut to_delete: Vec<Entity> = Vec::new();
    for (entity, other_level_position) in (&entities, &other_level_positions).join() {
        if entity == *player_entity { continue; }
        if other_level_position.index != index { continue; }
        positions.insert(entity, Position {
            x: other_level_position.x,
            y: other_level_position.y,
        }).expect("Unable to insert Position during thaw");
        to_delete.push(entity);
    }

    for delete in to_delete.iter() {
        other_level_positions.remove(*delete);
    }
}

/// Creates a very simple map and populates it with some very simple monsters.
pub fn generate_floor(seed: u64, floor_index: u32, world: &mut World) -> Map {
    let mut rng = RandomNumberGenerator::seeded(seed + (floor_index as u64));
    if let Some(existing_rng) = world.try_fetch::<RandomNumberGenerator>() {
        rng = existing_rng.deref().clone();
    } else {
        world.insert(rng.clone());
    }

    let map = match floor_index {
        0 => {
            let map = Map::new_map_oakwood(&mut rng, MapOptions {
                index: floor_index,
                width: 80,
                height: 40,
                has_upstairs: false,
                has_downstairs: true,
                has_debris: false,
            });
            spawn_npc_merchant(world, 40, 10);
            spawn_npc_captain(world, 60, 25);
            map
        }
        _ => {
            let map = Map::new_map_dynamic_rooms_and_corridors(&mut rng, MapOptions {
                index: floor_index,
                width: 100,
                height: 100,
                has_upstairs: floor_index != 0,
                has_downstairs: true,
                has_debris: true,
            });
            for (_index, room) in map.rooms.iter().skip(1).enumerate() {
                spawn_weighted_item(world, floor_index, room);
                spawn_weighted_monster(world, floor_index, room);
            }
            map
        }
    };
    let map_copy = map.clone();

    let (player_x, player_y) = map.idx_xy(map.player_spawn_index.expect("No player spawn index"));

    // Update the player position to ensure that existing entities are relocated
    {
        let entities = world.entities();
        let players = world.read_storage::<Player>();
        let mut positions = world.write_storage::<Position>();
        if let Some((_entity, _player, position)) =
            (&entities, &players, &mut positions).join().next()
        {
            position.x = player_x;
            position.y = player_y;
        }
    }

    world.insert(RunState::AwaitingInput);
    world.insert(map);
    world.insert(Point::new(player_x, player_y));

    if !world.has_value::<Entity>() {
        let player = spawn_player(world, player_x, player_y);
        world.insert(player);
    }

    return map_copy;
}
