use ratatui::style::Color;
use specs::prelude::*;

use crate::{Damage, Name, Player, Stats, component::{Experience, Monster, Position}, generate::map::Map, logbook::logbook::Logger};

pub struct DamageSystem {}

impl <'a> System<'a> for DamageSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Stats>,
        WriteStorage<'a, Damage>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Experience>,
        WriteExpect<'a, Map>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut stats,
            mut damage,
            positions,
            mut experience,
            mut map,
        ) = data;

        for (entity, stats, damage) in (&entities, &mut stats, &damage).join() {
            stats.hp.current -= damage.amount.iter().sum::<i32>();

            if stats.hp.current <= 0 && damage.attacker.is_some() {
                Experience::new(&mut experience, damage.attacker.unwrap(), stats.level * 100);
            }

            /*
             * Render bloodstains anywhere damage occurred
             */
            if let Some(pos) = positions.get(entity) {
                let index = map.xy_idx(pos.x, pos.y);
                map.bloodstains.insert(index);
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
        let monsters = ecs.read_storage::<Monster>();
        let player_entity = ecs.fetch::<Entity>();
        for (entity, stats, name) in (&entities, &stats, &names).join() {
            if stats.hp.current <= 0 {
                Logger::new()
                    .with_color(
                        if monsters.contains(entity) { Color::Red }
                        else if entity == *player_entity { Color::Green }
                        else { Color::Gray }
                    )
                    .append(format!("{} ", name.name))
                    .with_color(Color::White)
                    .append("has died.")
                    .log();
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
        if stats.hp.current <= 0 {
            return true;
        }
    }
    return false;
}
