use rltk::{Point};
use specs::prelude::*;

use crate::{map::{xy_idx, Map, MAX_WIDTH}, Logbook, Monster, Position, Viewshed};

pub struct MonsterSystem {

}

impl<'a> System<'a> for MonsterSystem {
    type SystemData = (
        ReadStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Monster>,
        ReadExpect<'a, Point>,
        WriteExpect<'a, Logbook>,
        WriteExpect<'a, Map>,
    );

    fn run(&mut self, (viewshed, mut position, monster, player_position, mut _logbook, mut map): Self::SystemData) {
        for (viewshed, position, _monster) in (&viewshed, &mut position, &monster).join() {
            if viewshed.visible_tiles.contains(&*player_position) {
                // logbook.entries.push(format!("Monster at ({}, {}) sees you!", position.x, position.y));
                let path = rltk::a_star_search(
                    xy_idx(position.x, position.y),
                    xy_idx(player_position.x, player_position.y),
                    &mut *map,
                );

                /*
                 * Move the monster towards the player, if possible
                 */
                if path.success && path.steps.len() > 1 {
                    let next_pos_x = path.steps[1] as i32 % MAX_WIDTH;
                    let next_pos_y = path.steps[1] as i32 / MAX_WIDTH;

                    let is_blocked_tile = map.blocked_tiles[xy_idx(next_pos_x, next_pos_y)];
                    if !is_blocked_tile {
                        position.x = next_pos_x;
                        position.y = next_pos_y;
                    }
                }
            }
        }
    }
}
