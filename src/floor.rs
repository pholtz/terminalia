use crate::{map::Map, BlocksTile, Inventory, Logbook, Monster, Name, Player, Position, Renderable, RunState, Stats, Viewshed};
use ratatui::style::Color;
use rltk::{Point, RandomNumberGenerator};
use specs::prelude::*;

/**
 * Creates a very simple map and populates it with some very simple monsters.
 */
pub fn generate_floor(seed: u64, floor_index: u8, world: &mut World) {
    let mut rng = RandomNumberGenerator::seeded(seed);
    let map = Map::new_map_dynamic_rooms_and_corridors(&mut rng);

    /*
     * Add the player character
     */
    let (player_x, player_y) = map.rooms[0].center();
    let player = world
        .create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: '@',
            bg: Color::Black,
            fg: Color::Yellow,
        })
        .with(Player {})
        .with(Name {
            name: "player".to_string(),
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
        })
        .with(BlocksTile {})
        .with(Stats {
            max_hp: 50,
            hp: 50,
            strength: 5,
            defense: 1,
        })
        .with(Inventory { gold: 0 })
        .build();

    for room in map.rooms.iter().skip(1) {
        let (monster_glyph, name) = match rng.roll_dice(1, 2) {
            1 => ('r', "rat"),
            2 => ('s', "snake"),
            _ => ('?', "???"),
        };
        world
            .create_entity()
            .with(Position {
                x: room.center().0,
                y: room.center().1,
            })
            .with(Renderable {
                glyph: monster_glyph,
                bg: Color::Black,
                fg: Color::Red,
            })
            .with(Monster {})
            .with(Name {
                name: name.to_string(),
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
            })
            .with(BlocksTile {})
            .with(Stats {
                max_hp: 6,
                hp: 6,
                strength: 2,
                defense: 1,
            })
            .build();
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
