use specs::prelude::*;
use specs_derive::Component;

use crate::base::Position;

#[derive(Component)]
pub struct RandomMover {}

#[derive(Default)]
pub struct DeltaTime(pub f32);

pub struct RandomWalkerSystem {
    accumulator: f32,
}

impl RandomWalkerSystem {
    pub fn new() -> Self {
        Self { accumulator: 0.0 }
    }
}

impl<'a> System<'a> for RandomWalkerSystem {
    type SystemData = (ReadStorage<'a, RandomMover>, WriteStorage<'a, Position>, Read<'a, DeltaTime>);

    fn run(&mut self, (entity, mut pos, delta): Self::SystemData) {
        self.accumulator += delta.0;
        println!("{} {}", delta.0, self.accumulator);
        if self.accumulator < 1.0 {
            return;
        }
        self.accumulator -= 1.0;
        
        let mut rng = rltk::RandomNumberGenerator::new();
        for (_entity, pos) in (&entity, &mut pos).join() {
            let direction = rng.roll_dice(1, 4);
            match direction {
                1 => pos.y -= 1,
                2 => pos.x += 1,
                3 => pos.y += 1,
                4 => pos.x -= 1,
                _ => println!("ahhhh"),
            };
            if pos.x < 0 {
                pos.x = 0;
            }
            if pos.y < 0 {
                pos.y = 0;
            }
            if pos.x > 79 {
                pos.x = 79;
            }
            if pos.y > 49 {
                pos.y = 49;
            }
        }
    }
}