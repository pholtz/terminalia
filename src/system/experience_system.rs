use ratatui::style::Color;
use specs::prelude::*;

use crate::{
    component::{Experience, Name, Pool, Stats},
    logbook::logbook::Logger,
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

        for (_entity, name, stat, experience) in (&entities, &names, &mut stats, &experience).join()
        {
            let exp = stat.exp.current + experience.amount.iter().sum::<i32>();
            if exp >= stat.exp.max {
                stat.exp = Pool {
                    current: exp - stat.exp.max,
                    max: stat.level * 1_000,
                };
                stat.level += 1;
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
