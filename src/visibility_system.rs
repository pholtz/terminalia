use rltk::{field_of_view, Point};
use specs::prelude::*;

use crate::{map::{xy_idx, Map}, Logbook, Player, Position, Viewshed};

pub struct VisibilitySystem {

}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, Map>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, Logbook>,
    );

    fn run(&mut self, (entities, mut map, player, mut viewshed, position, mut logbook): Self::SystemData) {
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
                        map.revealed_tiles[xy_idx(tile.x, tile.y)] = true;
                    }
                },
                None => {},
            }
        }
    }
}
