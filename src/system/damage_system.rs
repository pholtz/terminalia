use specs::prelude::*;

use crate::{Damage, Logbook, Name, Player, Stats, component::Position, generate::map::{Map, xy_idx}};

pub struct DamageSystem {}

impl <'a> System<'a> for DamageSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Stats>,
        WriteStorage<'a, Damage>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, Map>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut stats,
            mut damage,
            positions,
            mut map,
        ) = data;

        for (entity, stats, damage) in (&entities, &mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();

            /*
             * Render bloodstains anywhere damage occurred
             */
            if let Some(pos) = positions.get(entity) {
                map.bloodstains.insert(xy_idx(pos.x, pos.y));
            }
        }
        damage.clear();
    }
}

pub fn cleanup_dead_entities(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    {
        let entities = ecs.entities();
        let stats = ecs.read_storage::<Stats>();
        let names = ecs.read_storage::<Name>();
        let mut logbook = ecs.write_resource::<Logbook>();
        for (entity, stats, name) in (&entities, &stats, &names).join() {
            if stats.hp <= 0 {
                logbook.entries.push(format!("{} has died", name.name));
                dead.push(entity);
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to cleanup dead entity");
    }
}

pub fn is_game_over(ecs: &mut World) -> bool {
    let players = ecs.read_storage::<Player>();
    let stats = ecs.read_storage::<Stats>();
    for (_player, stats) in (&players, &stats).join() {
        if stats.hp <= 0 {
            return true;
        }
    }
    return false;
}
