use ratatui::style::Color;
use rltk::{Point, RandomNumberGenerator, field_of_view};
use specs::prelude::*;

use crate::{Player, Position, Viewshed, component::{Hidden, Name}, generate::map::Map, logbook::logbook::Logger};

pub struct VisibilitySystem {

}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, Map>,
        WriteExpect<'a, RandomNumberGenerator>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Hidden>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut map,
            mut rng,
            player,
            names,
            mut viewshed,
            position,
            mut hidden,
        ) = data;

        for (entity, viewshed, position) in (&entities, &mut viewshed, &position).join() {
            viewshed.visible_tiles.clear();
            viewshed.visible_tiles = field_of_view(
                Point::new(position.x, position.y),
                viewshed.range,
                &*map
            );
            viewshed.visible_tiles.retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

            /*
             * If this is the player's viewshed...
             * Iterate over all of the tiles in the viewshed and ensure they are all marked as revealed.
             * Later, during rendering, we'll use this to determine which tiles to render.
             */
            match player.get(entity) {
                Some(_) => {
                    for tile in viewshed.visible_tiles.iter() {
                        let index = map.xy_idx(tile.x, tile.y);
                        map.revealed_tiles[index] = true;

                        /*
                         * If the given tile contains hidden items, roll to reveal them.
                         * Otherwise, they will remain hidden until triggered.
                         */
                        for tile_entity in map.tile_content[index].iter() {
                            if let Some(_hidden) = hidden.get(*tile_entity) {
                                if rng.roll_dice(1, 20) == 20 {
                                    let name = names.get(*tile_entity).expect("Unable to fetch name for hidden entity");
                                    Logger::new()
                                        .append("Your keen gaze revealed a hidden ")
                                        .with_color(Color::Red)
                                        .append(format!("{}!", name.name))
                                        .log();
                                    // logbook.entries.push(format!("Your keen gaze revealed a hidden {}!", name.name));
                                    hidden.remove(*tile_entity);
                                }
                            }
                        }
                    }
                },
                None => {},
            }
        }
    }
}
