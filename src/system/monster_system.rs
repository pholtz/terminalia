use log::info;
use rltk::{Point};
use specs::prelude::*;

use crate::{map::{xy_idx, Map, MAX_WIDTH}, Attack, Logbook, Monster, Position, RunState, Viewshed};

pub struct MonsterSystem {

}

impl<'a> System<'a> for MonsterSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Attack>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, Logbook>,
        WriteExpect<'a, Map>,
        ReadExpect<'a, RunState>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            viewshed,
            mut position,
            monster,
            mut attack,
            player_position,
            player_entity,
            mut _logbook,
            mut map,
            runstate,
        ) = data;

        match *runstate {
            RunState::AwaitingInput | RunState::PlayerTurn | RunState::Descending => { return },
            RunState::MonsterTurn => {},
        }
        /*
         * Not sure why we still need this,
         * but it seems like whenever we remove it monsters stop being able to move.
         */
        map.populate_blocked();

        for (entity, viewshed, position, _monster) in (&entities, &viewshed, &mut position, &monster).join() {
            if viewshed.visible_tiles.contains(&*player_position) {
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
                    let is_player_tile = next_pos_x == player_position.x && next_pos_y == player_position.y;
                    if !is_blocked_tile && !is_player_tile {
                        position.x = next_pos_x;
                        position.y = next_pos_y;
                    }
                }

                /*
                 * Attack the player, if close enough
                 */
                let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(position.x, position.y), *player_position);
                if distance < 1.5 {
                    attack.insert(entity, Attack { target: *player_entity })
                        .expect("Unable to add monster attack");
                }
            }
        }
    }
}
