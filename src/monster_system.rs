use rltk::{field_of_view, Point};
use specs::prelude::*;

use crate::{map::{xy_idx, Map}, Logbook, Monster, Player, Position, Viewshed};

pub struct MonsterSystem {

}

impl<'a> System<'a> for MonsterSystem {
    type SystemData = (
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Monster>,
        ReadExpect<'a, Point>,
        WriteExpect<'a, Logbook>,
    );

    fn run(&mut self, (viewshed, position, monster, player_position, mut logbook): Self::SystemData) {
        for (viewshed, position, monster) in (&viewshed, &position, &monster).join() {
            if viewshed.visible_tiles.contains(&*player_position) {
                logbook.entries.push(format!("Monster at ({}, {}) sees you!", position.x, position.y));
            }
        }
    }
}
