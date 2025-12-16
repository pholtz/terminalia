use std::{collections::VecDeque, sync::Mutex};

use lazy_static::lazy_static;
use specs::prelude::*;

use crate::{App, RunState, Screen};

lazy_static! {
    pub static ref EFFECT_QUEUE: Mutex<VecDeque<Effect>> = Mutex::new(VecDeque::new());
}

pub enum EffectType {
    LevelUp { level: i32 },
}

pub struct Effect {
    pub effect_type: EffectType,
    pub creator: Option<Entity>,
}

pub fn create_effect(effect: Effect) {
    EFFECT_QUEUE.lock().unwrap().push_back(effect);
}

pub fn process_effects(app: &mut App) {
    loop {
        match EFFECT_QUEUE.lock().unwrap().pop_front() {
            Some(effect) => {
                match effect.effect_type {
                    /*
                     * __Level Up__
                     * Leveling up requires input from the user for attribute increases,
                     * and in the future we may want to include additional menu states
                     * as well, e.g. skill dialogs, backstory.
                     * To handle this, we need to go outside of the ecs so that we can
                     * modify core `App` fields. This lets us force a screen/state change.
                     */
                    EffectType::LevelUp { level: _ } => {
                        app.screen = Screen::Inventory;
                        app.runstate = RunState::LevelUp { index: 0 };
                    }
                }
            },
            None => break,
        }
    }
}
