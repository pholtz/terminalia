use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::style::Color;
use specs::prelude::*;

use crate::{component::{Damage, Hidden, Lifetime, Name, Position, Renderable, Stats, Triggerable}, generate::map::Map, logbook::logbook::Logger};
pub struct TriggerSystem {

}

impl<'a> System<'a> for TriggerSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Map>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Stats>,
        ReadStorage<'a, Triggerable>,
        WriteStorage<'a, Hidden>,
        WriteStorage<'a, Damage>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, Lifetime>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            map,
            mut positions,
            names,
            stats,
            triggerables,
            mut hidden,
            mut damages,
            mut renderables,
            mut lifetimes,
        ) = data;

        let mut particles_to_create: Vec<Position> = Vec::new();
        let mut entities_to_remove: Vec<Entity> = Vec::new();

        for (entity, position, name, _stats) in (&entities, &mut positions, &names, &stats).join() {
            let index = map.xy_idx(position.x, position.y);
            for colocated_entity in map.tile_content[index].iter() {
                if let Some(trigger) = triggerables.get(*colocated_entity) {
                    let trigger_name = names.get(*colocated_entity).expect("Unable to get name for triggerable");
                    Logger::new()
                        .with_color(Color::Blue)
                        .append(format!("{} ", name.name))
                        .with_color(Color::White)
                        .append("triggered ")
                        .with_color(Color::Blue)
                        .append(format!("{}, ", trigger_name.name))
                        .with_color(Color::White)
                        .append("dealing ")
                        .with_color(Color::Red)
                        .append(format!("{} damage!", trigger.damage))
                        .log();
                    Damage::new_damage(&mut damages, entity, trigger.damage);
                    hidden.remove(*colocated_entity);
                    particles_to_create.push(position.clone());
                    entities_to_remove.push(*colocated_entity);
                }
            }
        }

        /*
         * Create damage particles representing the triggered item.
         */
        for position in particles_to_create.iter() {
            entities.build_entity()
                .with(position.clone(), &mut positions)
                .with(Renderable { glyph: '!', fg: Color::LightRed, bg: Color::Gray, index: 0 }, &mut renderables)
                .with(Lifetime {
                    created_at: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("uhhhh")
                        .as_millis(),
                    lifetime_ms: 200,
                }, &mut lifetimes)
                .build();
        }

        entities_to_remove.iter().for_each(|entity| { entities.delete(*entity).expect("Unable to remove triggered entity"); });
    }
}
