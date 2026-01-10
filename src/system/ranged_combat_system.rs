use rltk::{Point, line2d};
use specs::prelude::*;

use crate::{
    component::{Equipped, Monster, Name, Position, RangedWeapon},
    generate::map::{Map, TileType},
};

pub struct RangedCombatSystem {}

impl<'a> System<'a> for RangedCombatSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Map>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, RangedWeapon>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            map,
            player_entity,
            positions,
            _names,
            _equipment,
            _monsters,
            mut ranged_weapons,
        ) = data;

        let player_position = positions
            .get(*player_entity)
            .expect("Unable to access player position");

        // TODO: Also should remove targeting for out of range entities.
        /*
         * Remove targeting for entities which are no longer in ecs.
         */
        for ranged in (&mut ranged_weapons).join() {
            match ranged.target {
                Some(target) => {
                    let mut distance: f32 = 0.0;
                    let mut has_los = false;
                    if let Some(monster_pos) = positions.get(target) {
                        distance = rltk::DistanceAlg::Pythagoras.distance2d(
                            Point {
                                x: player_position.x,
                                y: player_position.y,
                            },
                            Point {
                                x: monster_pos.x,
                                y: monster_pos.y,
                            },
                        );
                        has_los = has_line_of_sight(
                            &map,
                            Point {
                                x: player_position.x,
                                y: player_position.y,
                            },
                            Point {
                                x: monster_pos.x,
                                y: monster_pos.y,
                            },
                        );
                    }
                    if !entities.is_alive(target) || distance > ranged.range as f32 || !has_los {
                        ranged.target = None;
                    }
                }
                None => {}
            }
        }
    }
}

pub fn get_eligible_ranged_tiles(map: &Map, player_pos: &Point, range: i32) -> Vec<usize> {
    let mut eligible_tiles = Vec::new();
    for (index, tile) in map.tiles.iter().enumerate() {
        match *tile {
            TileType::Floor | TileType::DownStairs | TileType::UpStairs => {}
            _ => continue,
        }
        let (tile_x, tile_y) = map.idx_xy(index);
        let tile_pos = Point {
            x: tile_x,
            y: tile_y,
        };
        let distance_to_tile = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, tile_pos);
        if distance_to_tile <= range as f32 {
            if has_line_of_sight(map, *player_pos, tile_pos) {
                eligible_tiles.push(index);
            }
        }
    }
    return eligible_tiles;
}

pub fn has_line_of_sight(map: &Map, from: Point, to: Point) -> bool {
    return line2d(rltk::LineAlg::Bresenham, from, to)
        .iter()
        .map(|point| map.xy_idx(point.x, point.y))
        .all(|index| {
            matches!(
                map.tiles[index],
                TileType::Floor | TileType::DownStairs | TileType::UpStairs
            )
        });
}
