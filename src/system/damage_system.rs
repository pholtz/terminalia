use specs::prelude::*;

use crate::{Damage, Logbook, Name, Player, Stats};

pub struct DamageSystem {}

impl <'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, Stats>,
        WriteStorage<'a, Damage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut stats,
            mut damage
        ) = data;

        for (stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
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
