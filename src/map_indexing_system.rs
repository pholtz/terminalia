use specs::prelude::*;

use crate::{map::{xy_idx, Map}, BlocksTile, Position};

pub struct MapIndexingSystem {}

/**
 * This system keeps various map indices up to date.
 * 
 * These are generally done for performance and convenience,
 * so that we do not need to excessively loop over entities in the code
 * and extract their positions to do checks. Instead, we can quickly
 * check the desired position against the relevant map index and see
 * if there is anything present.
 */
impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        WriteExpect<'a, Map>,
        Entities<'a>,
    );
    
    // TODO: This allows monsters and players to collide if they move towards each other from 2 spaces apart
    fn run(&mut self, (position, blocks_tile, mut map, entities): Self::SystemData) {
        map.populate_blocked();
        map.clear_tile_content();
        for (entity, position) in (&entities, &position).join() {
            let index = xy_idx(position.x, position.y);

            // Keep the `blocked_tiles` index up to date
            let _p: Option<&BlocksTile> = blocks_tile.get(entity);
            if let Some(_p) = _p {
                map.blocked_tiles[index] = true;
            }
            
            // Keep the `tile_content` index up to date
            map.tile_content[index].push(entity);
        }
    }
}
