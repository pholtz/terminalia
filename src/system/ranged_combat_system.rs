use rltk::{Point, line2d};
use specs::prelude::*;

use crate::{component::{EquipmentSlot, Equipped, Monster, Name, Position, RangedWeapon}, generate::map::{Map, TileType}};

pub struct RangedCombatSystem {}

impl<'a> System<'a> for RangedCombatSystem {
    type SystemData = (
        Entities<'a>,
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
            player_entity,
            positions,
            names,
            equipment,
            monsters,
            mut ranged_weapons,
        ) = data;

        let player_position = positions.get(*player_entity).expect("Unable to access player position");

        // TODO: Also should remove targeting for out of range entities.
        /*
         * Remove targeting for entities which are no longer in ecs.
         */
        for ranged in (&mut ranged_weapons).join() {
            match ranged.target {
                Some(target) => {
                    if !entities.is_alive(target) { ranged.target = None; }
                },
                None => {}
            }
        }
    }
}

pub fn get_eligible_ranged_tiles(map: &Map, player_pos: &Point, range: i32) -> Vec<usize> {
    let mut eligible_tiles = Vec::new();
    for (index, tile) in map.tiles.iter().enumerate() {
        match *tile {
            TileType::Floor | TileType::DownStairs | TileType::UpStairs => {},
            _ => continue,
        }
        let (tile_x, tile_y) = map.idx_xy(index);
        let tile_pos = Point { x: tile_x, y: tile_y };
        let distance_to_tile = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, tile_pos);
        if distance_to_tile <= range as f32 {
            let has_line_of_sight = line2d(rltk::LineAlg::Bresenham, *player_pos, tile_pos).iter()
                .map(|point| map.xy_idx(point.x, point.y))
                .all(|index| matches!(map.tiles[index], TileType::Floor | TileType::DownStairs | TileType::UpStairs));
            if has_line_of_sight {
                eligible_tiles.push(index);
            }
        }
    }
    return eligible_tiles;
}

pub fn with_world<R>(world: &mut World, f: impl FnOnce(&mut World) -> R) -> R {
    f(world)
}

pub fn get_player_ranged_weapon_entity(ecs: &mut World) -> Option<Entity> {
    let entities = ecs.entities();
    let player_entity = ecs.fetch::<Entity>();
    let equipped = ecs.read_storage::<Equipped>();
    let ranged_weapons = ecs.read_storage::<RangedWeapon>();

    for (entity, equipped, _ranged_weapon) in (&entities, &equipped, &ranged_weapons).join() {
        if equipped.slot == EquipmentSlot::Weapon && equipped.owner == *player_entity {
            return Some(entity)
        }
    }
    return None
}
