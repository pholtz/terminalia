use crate::{
    component::InBackpack, generate::{
        map::Map,
        spawn::{spawn_player, spawn_weighted_item, spawn_weighted_monster},
    }, Logbook, Player, Position, RunState
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
pub fn generate_floor(seed: u64, floor_index: u8, world: &mut World) {
    let mut rng = RandomNumberGenerator::seeded(seed + (floor_index as u64));
    let map = Map::new_map_dynamic_rooms_and_corridors(&mut rng);

    // Add the player character or fetch them if they already exist
    let (player_x, player_y) = map.rooms[0].center();
    let (player, initializing) = if let Some(p) = world.try_fetch::<Entity>() {
        (*p, false)
    } else {
        (spawn_player(world, player_x, player_y), true)
    };

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

    for (_index, room) in map.rooms.iter().skip(1).enumerate() {
        spawn_weighted_item(world, seed, floor_index, room);
        spawn_weighted_monster(world, seed, floor_index, room);
    }

    world.insert(RunState::AwaitingInput);
    world.insert(map);
    world.insert(Point::new(player_x, player_y));
    world.insert(player);
    if initializing {
        world.insert(Logbook {
            entries: vec!["You begin your adventure in a smallish room...".to_string()],
            scroll_offset: 0,
        });
    }
}
