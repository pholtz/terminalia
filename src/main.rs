use std::{fs::File, io, time::Duration};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use log::LevelFilter;
use rand::Rng;
use ratatui::{DefaultTerminal, Frame};
use simplelog::{CombinedLogger, Config, WriteLogger};
use specs::prelude::*;

mod component;
mod floor;
mod input;
mod map;
mod rect;
mod render;
mod spawn;
mod system;

use input::menu::handle_menu_key_event;
use render::game_over::render_game_over;
use render::inventory::render_inventory;
use render::menu::render_menu;
use system::{
    damage_system, inventory_system, map_indexing_system, melee_combat_system, monster_system,
    visibility_system,
};

use crate::{
    component::{
        Attack, BlocksTile, Damage, InBackpack, Inventory, Item, Logbook, MagicMapper, Monster,
        Name, Player, Position, Potion, Renderable, Stats, Viewshed, WantsToConsumeItem,
        WantsToPickupItem,
    },
    damage_system::DamageSystem,
    floor::{generate_floor, reset_floor},
    input::{
        game_over::handle_game_over_key_event, main_explore::handle_main_explore_key_event,
        main_inventory::handle_main_inventory_key_event, main_log::handle_main_log_key_event,
    },
    inventory_system::InventorySystem,
    map_indexing_system::MapIndexingSystem,
    melee_combat_system::MeleeCombatSystem,
    monster_system::MonsterSystem,
    render::{game::render_game, log::render_log},
    visibility_system::VisibilitySystem,
};

#[derive(Debug)]
pub enum RootScreen {
    Menu,
    Main,
    GameOver,
}

#[derive(Debug)]
pub enum Screen {
    /**
     * The default view.
     * A split screen between the viewshed and the minilog.
     */
    Explore,

    /**
     * A toggleable view containing a fullscreen logbook.
     */
    Log,

    /**
     * A non-combat screen that shows the player's inventory
     * and allows them to use or drop inventory item.
     */
    Inventory,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RunState {
    AwaitingInput,
    PlayerTurn,
    MonsterTurn,
    Descending,
}

pub struct App {
    pub ecs: World,
    pub dispatcher: Dispatcher<'static, 'static>,
    root_screen: RootScreen,
    screen: Screen,
    menu_index: u8,
    floor_index: u8,
    exit: bool,
}

impl App {
    /**
     * The core game loop.
     *
     * Even though this is a turn based game, we render and run background systems continuously.
     * This allows us to perform animations, and ensures that systems have a chance to settle
     * after a key event and resulting state changes. For example, if the combat system removes
     * a monster after the map indexing system runs, we ensure that indexing will be rerun each
     * tick and thus will eventually settle, likely far before any further input occurs.
     *
     * This is somewhat inefficient since lots of things rerun that probably don't need to,
     * but it also really simplifies game logic and lets us think about systems as continuous.
     *
     * --- I N P U T  H A N D L I N G ---
     * The input event loop runs first via `handle_events()`. It polls briefly for any key events
     * and dispatches the requisite handlers, which mostly just write state to ecs for downstream
     * systems to handle. Returns true if a state transition eligible event (e.g. movement) occurred.
     */
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        terminal.draw(|frame| self.draw(frame))?;
        while !self.exit {
            let has_eligible_event = self.handle_events()?;
            match self.root_screen {
                RootScreen::Menu => {}
                RootScreen::GameOver => {}
                RootScreen::Main => {
                    let runstate: RunState;
                    {
                        runstate = *self.ecs.read_resource::<RunState>();
                    }

                    let mut next_runstate = runstate;
                    match runstate {
                        RunState::AwaitingInput => {
                            if has_eligible_event {
                                next_runstate = RunState::PlayerTurn;
                            }
                        }
                        RunState::PlayerTurn => next_runstate = RunState::MonsterTurn,
                        RunState::MonsterTurn => next_runstate = RunState::AwaitingInput,
                        RunState::Descending => {
                            self.floor_index += 1;
                            reset_floor(&mut self.ecs);
                            generate_floor(0, self.floor_index, &mut self.ecs);
                            next_runstate = RunState::AwaitingInput;
                        }
                    }

                    if runstate != next_runstate {
                        let mut runstate = self.ecs.write_resource::<RunState>();
                        *runstate = next_runstate;
                    }

                    self.dispatcher.dispatch(&self.ecs);
                    if damage_system::is_game_over(&mut self.ecs) {
                        self.root_screen = RootScreen::GameOver;
                    }
                    damage_system::cleanup_dead_entities(&mut self.ecs);
                }
            }
            self.ecs.maintain();
            terminal.draw(|frame| self.draw(frame))?;
            std::thread::sleep(Duration::from_millis(16));
        }
        Ok(())
    }

    /**
     * Root event handler for all screens.
     */
    fn handle_events(&mut self) -> io::Result<bool> {
        let mut is_eligible_event = false;
        if event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    is_eligible_event = self.handle_key_event(key_event);
                }
                _ => {}
            }
            return Ok(is_eligible_event);
        }
        return Ok(false);
    }

    /**
     * Base key handler for all screens.
     *
     * Returns a boolean with the following states:
     * true -> if the event should trigger a state transition (e.g. movement)
     * false -> if the event should not trigger a state transition (e.g. checking inventory)
     */
    fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        match self.root_screen {
            RootScreen::Menu => handle_menu_key_event(self, key_event),
            RootScreen::Main => match self.screen {
                Screen::Explore => handle_main_explore_key_event(self, key_event),
                Screen::Log => handle_main_log_key_event(self, key_event),
                Screen::Inventory => handle_main_inventory_key_event(self, key_event),
            },
            RootScreen::GameOver => handle_game_over_key_event(self, key_event),
        }
    }

    /**
     * Base renderer for all screens.
     * Delegates to the relevant subrenderer based on the given screen and state.
     */
    fn draw(&mut self, frame: &mut Frame) {
        match self.root_screen {
            RootScreen::Menu => render_menu(frame, self.menu_index),
            RootScreen::Main => match self.screen {
                Screen::Explore => render_game(&mut self.ecs, frame, self.floor_index),
                Screen::Log => render_log(&mut self.ecs, frame),
                Screen::Inventory => render_inventory(&mut self.ecs, frame),
            },
            RootScreen::GameOver => render_game_over(frame),
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

fn reinitialize_world() -> World {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Renderable>();
    world.register::<Player>();
    world.register::<Monster>();
    world.register::<Name>();
    world.register::<Viewshed>();
    world.register::<BlocksTile>();
    world.register::<Stats>();
    world.register::<Inventory>();
    world.register::<Attack>();
    world.register::<Damage>();
    world.register::<Item>();
    world.register::<Potion>();
    world.register::<MagicMapper>();
    world.register::<InBackpack>();
    world.register::<WantsToPickupItem>();
    world.register::<WantsToConsumeItem>();
    return world;
}

fn reinitialize_systems(world: &mut World) -> Dispatcher<'static, 'static> {
    let mut dispatcher = DispatcherBuilder::new()
        .with(VisibilitySystem {}, "visibility_system", &[])
        .with(InventorySystem {}, "inventory_system", &[])
        .with(MonsterSystem {}, "monster_system", &["visibility_system"])
        .with(
            MapIndexingSystem {},
            "map_indexing_system",
            &["monster_system"],
        )
        .with(
            MeleeCombatSystem {},
            "melee_combat_system",
            &["map_indexing_system"],
        )
        .with(DamageSystem {}, "damage_system", &["melee_combat_system"])
        .build();
    dispatcher.setup(world);
    return dispatcher;
}

fn main() -> Result<()> {
    color_eyre::install().expect("ahhhh");

    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        File::create("spear.log").unwrap(),
    )])
    .unwrap();

    let mut world = reinitialize_world();
    let dispatcher = reinitialize_systems(&mut world);
    generate_floor(rand::rng().random(), 0, &mut world);

    let mut terminal = ratatui::init();
    let app_result = App {
        ecs: world,
        dispatcher: dispatcher,
        root_screen: RootScreen::Menu,
        screen: Screen::Explore,
        menu_index: 0,
        floor_index: 0,
        exit: false,
    }
    .run(&mut terminal);
    ratatui::restore();
    return app_result;
}
