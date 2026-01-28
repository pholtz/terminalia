use ratatui::style::Color;
use specs::prelude::*;

use crate::{
    component::{Experience, Name, Pool, Stats}, effect::effect::{Effect, EffectType, create_effect}, logbook::logbook::Logger
};

pub struct ExperienceSystem {}

impl<'a> System<'a> for ExperienceSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Stats>,
        WriteStorage<'a, Experience>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, names, mut stats, mut experience) = data;

        for (entity, name, stat, experience) in (&entities, &names, &mut stats, &experience).join()
        {
            let exp = stat.exp.current + experience.amount.iter().sum::<i32>();
            if exp >= stat.exp.max {
                stat.exp = Pool {
                    current: exp - stat.exp.max,
                    max: stat.level * 1_000,
                };
                stat.level += 1;
                create_effect(Effect {
                    _creator: Some(entity),
                    effect_type: EffectType::LevelUp { _level: stat.level },
                });

                // Upgrade max hp and mp based on attributes
                let hp_multiplier = std::cmp::max(1, (stat.constitution - 10) / 2);
                stat.hp.max = stat.hp.max + (2 * hp_multiplier);
                stat.hp.current = stat.hp.max;
                let mp_multiplier = std::cmp::max(1, (stat.intelligence - 10) / 2);
                stat.mp.max = stat.mp.max + (2 * mp_multiplier);
                stat.mp.current = stat.mp.max;

                Logger::new()
                    .append_with_color(
                        Color::Yellow,
                        format!("{} has increased in experience to Level {}!", name.name, stat.level),
                    )
                    .log();
            } else {
                stat.exp = Pool {
                    current: exp,
                    max: stat.exp.max,
                };
            }
        }
        experience.clear();
    }
}
