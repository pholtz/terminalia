use rltk::{GameState, RGB, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

mod base;
mod map;
mod random_mover;

use base::*;

use crate::{map::{xy_idx, TileType}, random_mover::{DeltaTime, RandomMover, RandomWalkerSystem}};

struct State {
    pub ecs: World,
    pub dispatcher: Dispatcher<'static, 'static>,
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Vec<TileType>>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let dest = xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map[dest] != TileType::Wall {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
        }
    }
}

fn player_input(gs: &mut State, ctx: &mut Rltk) {
    // Player movement
    match ctx.key {
        None => {} // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::A => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right | VirtualKeyCode::D => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up | VirtualKeyCode::W => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down | VirtualKeyCode::S => try_move_player(0, 1, &mut gs.ecs),
            VirtualKeyCode::Escape => ::std::process::exit(0),
            _ => {}
        },
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        self.ecs.insert(DeltaTime(ctx.frame_time_ms / 1_000.0));

        player_input(self, ctx);
        self.dispatcher.dispatch(&mut self.ecs);
        self.ecs.maintain();
        // self.run_systems();

        let map = self.ecs.fetch::<Vec<map::TileType>>();
        map::draw_map(&map, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut random_walker = RandomWalkerSystem::new();
        random_walker.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    let mut gs = State {
        ecs: World::new(),
        dispatcher: DispatcherBuilder::new()
            .with(RandomWalkerSystem::new(), "random_walker", &[])
            .build()
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<RandomMover>();
    gs.ecs.register::<Player>();

    gs.ecs.insert(map::new_map());
    gs.ecs.insert(DeltaTime::default());

    gs.ecs
        .create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .build();

    for i in 0..10 {
        gs.ecs
            .create_entity()
            .with(Position { x: i * 7, y: 20 })
            .with(RandomMover{})
            .with(Renderable {
                glyph: rltk::to_cp437('☺'),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .build();
    }

    println!("Starting main loop!");
    rltk::main_loop(context, gs)
}
