use log::info;
use ratatui::style::Color;
use rltk::{Point, RandomNumberGenerator, field_of_view};
use specs::prelude::*;

use crate::{
    Player, Position, Viewshed,
    component::{EquipmentSlot, Equipped, Hidden, Monster, Name, RangedWeapon},
    generate::{map::Map, rect::Rect},
    logbook::logbook::Logger,
};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, Map>,
        WriteExpect<'a, RandomNumberGenerator>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Hidden>,
        ReadStorage<'a, Equipped>,
        WriteStorage<'a, RangedWeapon>,
        ReadStorage<'a, Monster>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut map,
            mut rng,
            player_entity,
            player,
            names,
            mut viewshed,
            position,
            mut hidden,
            equipped,
            mut ranged_weapon,
            monster,
        ) = data;

        for (entity, viewshed, pos) in (&entities, &mut viewshed, &position).join() {
            viewshed.visible_tiles.clear();
            viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
            viewshed
                .visible_tiles
                .retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

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
                                    let name = names
                                        .get(*tile_entity)
                                        .expect("Unable to fetch name for hidden entity");
                                    Logger::new()
                                        .append("Your keen gaze revealed a hidden ")
                                        .with_color(Color::Red)
                                        .append(format!("{}!", name.name))
                                        .log();
                                    hidden.remove(*tile_entity);
                                }
                            }
                        }
                    }
                }
                None => {}
            }

            /*
             * Targeting system
             * If the player is wielding a ranged weapon and does not already have an assigned target,
             * attempt to assign one. If no visible enemies are within range, then do nothing.
             *
             * Likely move this to it's own system or find a better place for it...
             */
            match player.get(entity) {
                Some(_) => {
                    for (item_entity, equipped, ranged) in
                        (&entities, &equipped, &mut ranged_weapon).join()
                    {
                        if equipped.owner != entity
                            || equipped.slot != EquipmentSlot::Weapon
                            || ranged.target.is_some()
                        {
                            continue;
                        }
                        for (monster_entity, _monster, monster_pos) in
                            (&entities, &monster, &position).join()
                        {
                            let distance = rltk::DistanceAlg::Pythagoras.distance2d(
                                Point { x: pos.x, y: pos.y },
                                Point {
                                    x: monster_pos.x,
                                    y: monster_pos.y,
                                },
                            );
                            if distance <= ranged.range as f32 {
                                info!(
                                    "{} takes aim at {} with a {}",
                                    names.get(entity).unwrap().name,
                                    names.get(monster_entity).unwrap().name,
                                    names.get(item_entity).unwrap().name
                                );
                                ranged.target = Some(monster_entity);
                            }
                        }
                    }
                }
                None => {}
            }
        }

        /*
         * Targeting system
         * If the player is wielding a ranged weapon and does not already have an assigned target,
         * attempt to assign one. If no visible enemies are within range, then do nothing.
         *
         * Likely move this to it's own system or find a better place for it...
         */
        let player_position = position.get(*player_entity).expect("Unable to access player position");

        for ranged in (&mut ranged_weapon).join() {
            match ranged.target {
                Some(target) => {
                    if !entities.is_alive(target) { ranged.target = None; }
                },
                None => {}
            }
        }

        for (item_entity, equipped, ranged) in (&entities, &equipped, &mut ranged_weapon).join() {
            if equipped.owner != *player_entity
                || equipped.slot != EquipmentSlot::Weapon
                || ranged.target.is_some()
            {
                continue;
            }
            for (monster_entity, _monster, monster_pos) in (&entities, &monster, &position).join() {
                let distance = rltk::DistanceAlg::Pythagoras.distance2d(
                    Point { x: player_position.x, y: player_position.y },
                    Point {
                        x: monster_pos.x,
                        y: monster_pos.y,
                    },
                );
                if distance <= ranged.range as f32 {
                    info!(
                        "{} takes aim at {} with a {}",
                        names.get(*player_entity).unwrap().name,
                        names.get(monster_entity).unwrap().name,
                        names.get(item_entity).unwrap().name
                    );
                    ranged.target = Some(monster_entity);
                }
            }
        }
    }
}

pub fn get_player_ranged_weapon_entity(ecs: &mut World) -> Option<Entity> {
    let entities = ecs.entities();
    let player_entity = ecs.fetch::<Entity>();
    let equipped = ecs.read_storage::<Equipped>();
    let mut ranged_weapons = ecs.write_storage::<RangedWeapon>();

    for (entity, equipped, _ranged_weapon) in (&entities, &equipped, &mut ranged_weapons).join() {
        if equipped.slot == EquipmentSlot::Weapon && equipped.owner == *player_entity {
            return Some(entity)
        }
    }
    return None
}
