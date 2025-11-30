use crate::{
    map::{Map, MAX_ITEMS, MAX_MONSTERS}, spawn::{spawn_dagger, spawn_monster_rat, spawn_monster_snake, spawn_player, spawn_potion_health, spawn_scroll_magic_mapping}, BlocksTile, Inventory, Logbook, Monster, Name, Player, Position, Renderable, RunState, Stats, Viewshed
};
use log::{warn};
use rltk::{Point, RandomNumberGenerator};
use specs::prelude::*;

/**
 * Creates a very simple map and populates it with some very simple monsters.
 */
pub fn generate_floor(seed: u64, floor_index: u8, world: &mut World) {
    let mut rng = RandomNumberGenerator::seeded(seed + (floor_index as u64));
    let map = Map::new_map_dynamic_rooms_and_corridors(&mut rng);

    /*
     * Add the player character
     */
    let (player_x, player_y) = map.rooms[0].center();
    let player = spawn_player(world, player_x, player_y);

    // Spawn monsters
    for (index, room) in map.rooms.iter().skip(1).enumerate() {
        if index > (MAX_MONSTERS as usize) { break; }
        match rng.roll_dice(1, 2) {
            1 => spawn_monster_rat(world, Position { x: room.center().0, y: room.center().1 }),
            2 => spawn_monster_snake(world, Position { x: room.center().0, y: room.center().1 }),
            _ => warn!("generate_floor received unexpected random monster spawn diceroll, skipping..."),
        };
    }

    // Spawn item
    for (index, room) in map.rooms.iter().skip(1).enumerate() {
        if index > (MAX_ITEMS as usize) { break; }
        let width = room.x2 - room.x1;
        let height = room.y2 - room.y1;
        let item_x = room.x1 + rng.roll_dice(1, width - 1);
        let item_y = room.y1 + rng.roll_dice(1, height - 1);
         match rng.roll_dice(1, 3) {
            1 => spawn_potion_health(world, item_x, item_y),
            2 => spawn_scroll_magic_mapping(world, item_x, item_y),
            3 => spawn_dagger(world, item_x, item_y),
            _ => warn!("generate_floor received unexpected random item spawn diceroll, skipping..."),
         }
    }

    world.insert(RunState::AwaitingInput);
    world.insert(map);
    world.insert(Point::new(player_x, player_y));
    world.insert(player);
    world.insert(Logbook {
        entries: vec!["You begin your adventure in a smallish room...".to_string()],
        scroll_offset: 0,
    });
}
