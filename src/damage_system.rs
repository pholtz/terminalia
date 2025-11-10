use specs::prelude::*;

use crate::{Damage, Stats};

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
        for (entity, stats) in (&entities, &stats).join() {
            if stats.hp <= 0 { dead.push(entity); }
        }
    }
    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to cleanup dead entity");
    }
}
