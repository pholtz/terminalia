use std::time::{SystemTime, UNIX_EPOCH};

use specs::prelude::*;

use crate::component::Lifetime;

pub struct ParticleSystem {

}

impl<'a> System<'a> for ParticleSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Lifetime>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            lifetimes
        ) = data;

        let current_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("uhhhh")
            .as_millis();

        let mut lifetimes_to_delete: Vec<Entity> = Vec::new();
        for (entity, lifetime) in (&entities, &lifetimes).join() {
            if (current_ms - lifetime.created_at) > lifetime.lifetime_ms {
                lifetimes_to_delete.push(entity);
            }
        }
        lifetimes_to_delete.iter().for_each(|lifetime| {
            entities.delete(*lifetime).expect("Unable to delete entity with expired lifetime");
        });
    }
}
