use std::ops::Deref;

use crate::{
    Player, Position, RunState, component::InBackpack, generate::{
        map::{Map, MapOptions}, spawn::{spawn_npc_captain, spawn_npc_merchant, spawn_player, spawn_weighted_item, spawn_weighted_monster}
    }
};
use rltk::{Point, RandomNumberGenerator};
use specs::prelude::*;

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

/// Creates a very simple map and populates it with some very simple monsters.
pub fn generate_floor(seed: u64, floor_index: u32, world: &mut World) {
    let mut rng = RandomNumberGenerator::seeded(seed + (floor_index as u64));
    if let Some(existing_rng) = world.try_fetch::<RandomNumberGenerator>() {
        rng = existing_rng.deref().clone();
    } else {
        world.insert(rng.clone());
    }

    let map = match floor_index {
        0 => {
            let map = Map::new_map_oakwood(&mut rng, MapOptions {
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
}
