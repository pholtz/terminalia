use std::{fs::File, io, time::Duration};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use log::LevelFilter;
use rand::Rng;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
};
use rltk::Point;
use simplelog::{CombinedLogger, Config, WriteLogger};
use specs::prelude::*;
use std::cmp::{max, min};

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
        Attack, BlocksTile, Damage, InBackpack, Inventory, Item, Logbook, MagicMapper, Monster, Name, Player, Position, Potion, Renderable, Stats, Viewshed, WantsToConsumeItem, WantsToPickupItem
    },
    damage_system::DamageSystem,
    floor::generate_floor,
    inventory_system::InventorySystem,
    map::{xy_idx, Map, TileType, MAX_HEIGHT, MAX_WIDTH},
    map_indexing_system::MapIndexingSystem,
    melee_combat_system::MeleeCombatSystem,
    monster_system::MonsterSystem,
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

#[derive(Debug)]
pub enum RunState {
    AwaitingInput,
    PlayerTurn,
    MonsterTurn,
}

fn try_get_item(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut logbook = ecs.fetch_mut::<Logbook>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => logbook
            .entries
            .push("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(
                    *player_entity,
                    WantsToPickupItem {
                        collected_by: *player_entity,
                        item: item,
                    },
                )
                .expect("Unable to insert item pickup into ecs");
        }
    }
    return true;
}

fn try_consume_item(ecs: &mut World) -> bool {
    let player_entity = ecs.fetch::<Entity>();
    let inventories = ecs.read_storage::<Inventory>();
    let mut wants_consume = ecs.write_storage::<WantsToConsumeItem>();

    if let Some(inventory) = inventories.get(*player_entity) {
        if let Some(item_stack) = inventory.items.get_index(inventory.index) {
            if let Some(item) = item_stack.1.get(0) {
                wants_consume
                    .insert(*player_entity, WantsToConsumeItem { item: *item })
                    .expect("Unable to insert item consumption into ecs");
            }
        }
    }
    return true;
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) -> bool {
    let entities = ecs.entities();
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut attacks = ecs.write_storage::<Attack>();
    let stats = ecs.read_storage::<Stats>();
    let mut player_position = ecs.write_resource::<Point>();
    let map = ecs.fetch::<Map>();
    let mut _logbook = ecs.write_resource::<Logbook>();

    for (entity, pos, _player) in (&entities, &mut positions, &mut players).join() {
        let next_pos_x = min(MAX_WIDTH - 1, max(0, pos.x + delta_x));
        let next_pos_y = min(MAX_HEIGHT - 1, max(0, pos.y + delta_y));
        let dest = xy_idx(pos.x + delta_x, pos.y + delta_y);

        for target in map.tile_content[dest].iter() {
            let target_stats = stats.get(*target);
            match target_stats {
                None => {}
                Some(_t) => {
                    attacks
                        .insert(entity, Attack { target: *target })
                        .expect("Unable to add attack");
                    return true;
                }
            }
        }

        let is_blocked_tile = map.blocked_tiles[dest];
        if !is_blocked_tile {
            pos.x = next_pos_x;
            pos.y = next_pos_y;
            player_position.x = next_pos_x;
            player_position.y = next_pos_y;
        }
    }
    return true;
}

fn try_scroll_logbook(ecs: &mut World, delta: i16) -> bool {
    let mut logbook = ecs.write_resource::<Logbook>();
    if delta.is_positive() {
        match logbook.scroll_offset.checked_add(delta as u16) {
            Some(offset) => {
                if offset <= ((logbook.entries.len() - 1) as u16) {
                    logbook.scroll_offset = offset
                }
            }
            None => {}
        }
    } else {
        match logbook.scroll_offset.checked_sub(delta.abs() as u16) {
            Some(offset) => logbook.scroll_offset = offset,
            None => {}
        }
    }
    return false;
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
                    {
                        let mut runstate = (&mut self.ecs).write_resource::<RunState>();
                        match *runstate {
                            RunState::AwaitingInput => {
                                if has_eligible_event {
                                    *runstate = RunState::PlayerTurn;
                                }
                            }
                            RunState::PlayerTurn => *runstate = RunState::MonsterTurn,
                            RunState::MonsterTurn => *runstate = RunState::AwaitingInput,
                        }
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

    fn draw(&mut self, frame: &mut Frame) {
        match self.root_screen {
            RootScreen::Menu => render_menu(frame, self.menu_index),
            RootScreen::Main => match self.screen {
                Screen::Explore => self.render_game(frame),
                Screen::Log => self.render_log(frame),
                Screen::Inventory => render_inventory(&mut self.ecs, frame),
            },
            RootScreen::GameOver => render_game_over(frame),
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

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
            RootScreen::Menu => return handle_menu_key_event(self, key_event),

            RootScreen::Main => match self.screen {
                Screen::Explore => match key_event.code {
                    KeyCode::Esc => {
                        self.root_screen = RootScreen::Menu;
                        return false;
                    }

                    KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('h') => {
                        try_move_player(-1, 0, &mut self.ecs)
                    }

                    KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('l') => {
                        try_move_player(1, 0, &mut self.ecs)
                    }

                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
                        try_move_player(0, -1, &mut self.ecs)
                    }

                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
                        try_move_player(0, 1, &mut self.ecs)
                    }

                    KeyCode::Char('g') => try_get_item(&mut self.ecs),
                    KeyCode::Char('q') => {
                        self.screen = Screen::Log;
                        return false;
                    }
                    KeyCode::Char('i') => {
                        self.screen = Screen::Inventory;
                        return false;
                    }
                    _ => false,
                },
                Screen::Log => match key_event.code {
                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
                        try_scroll_logbook(&mut self.ecs, -1)
                    }

                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
                        try_scroll_logbook(&mut self.ecs, 1)
                    }

                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.screen = Screen::Explore;
                        return false;
                    }
                    _ => false,
                },
                Screen::Inventory => match key_event.code {
                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
                        let player_entity = self.ecs.fetch::<Entity>();
                        let mut inventories = self.ecs.write_storage::<Inventory>();
                        if let Some(inventory) = inventories.get_mut(*player_entity) {
                            if inventory.index > 0 {
                                inventory.index -= 1;
                            }
                        }
                        return false;
                    }

                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
                        let player_entity = self.ecs.fetch::<Entity>();
                        let mut inventories = self.ecs.write_storage::<Inventory>();
                        if let Some(inventory) = inventories.get_mut(*player_entity) {
                            if inventory.index + 1 < inventory.items.len() {
                                inventory.index += 1;
                            }
                        }
                        return false;
                    }

                    KeyCode::Char('i') | KeyCode::Esc => {
                        self.screen = Screen::Explore;
                        return false;
                    }
                    KeyCode::Enter => {
                        try_consume_item(&mut self.ecs);
                        self.screen = Screen::Explore;
                        return true;
                    }
                    _ => false,
                },
            },

            RootScreen::GameOver => match key_event.code {
                KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Esc => {
                    self.ecs = reinitialize_world();
                    self.dispatcher = reinitialize_systems(&mut self.ecs);
                    generate_floor(0, 0, &mut self.ecs);
                    self.root_screen = RootScreen::Menu;
                    return false;
                }
                _ => false,
            },
        }
    }

    /**
     * The base render function for the game itself.
     *
     * This should handle rendering the game window itself
     * as well as the log and any other status windows we might need.
     *
     * Game objects themselves should be derived from ecs.
     */
    fn render_game(&self, frame: &mut Frame) {
        /*
         * Create the base map lines and spans to render the main game split
         */
        let map = self.ecs.fetch::<Map>();
        let mut lines = Vec::new();
        let mut spans = Vec::new();
        for (index, tile) in map.tiles.iter().enumerate() {
            if map.revealed_tiles[index] {
                match tile {
                    TileType::Floor => {
                        spans.push(Span::styled(".", Style::default().fg(Color::Gray)))
                    }
                    TileType::Wall => {
                        spans.push(Span::styled("#", Style::default().fg(Color::Green)))
                    }
                    TileType::DownStairs => {
                        spans.push(Span::styled("目", Style::default().fg(Color::Yellow)))
                    }
                }
            } else {
                spans.push(Span::styled(" ", Style::default()));
            }

            if (index + 1) % (MAX_WIDTH as usize) == 0 {
                lines.push(Line::from(spans));
                spans = Vec::new();
            }
        }

        /*
         * Overwrite base map spans with any renderable characters.
         * Sort renderables by index (render order) prior to rendering, lowest first.
         */
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let mut renderable_entities = (&positions, &renderables).join().collect::<Vec<_>>();
        renderable_entities.sort_by(|&a, &b| b.1.index.cmp(&a.1.index));
        for (pos, render) in renderable_entities.iter() {
            if map.revealed_tiles[xy_idx(pos.x, pos.y)] {
                lines[pos.y as usize].spans[pos.x as usize] =
                    Span::styled(render.glyph.to_string(), Style::default().fg(render.fg).bg(render.bg));
            }
        }

        /*
         * Format the status bar with health, gold, etc.
         */
        let player = self.ecs.fetch::<Entity>();
        let stats = self.ecs.read_storage::<Stats>();
        let inventory = self.ecs.read_storage::<Inventory>();
        let status_line = match (stats.get(*player), inventory.get(*player)) {
            (Some(stats), Some(inventory)) => format!(
                "HP: {} / {}  Floor: {}  Gold: {}",
                stats.hp, stats.max_hp, self.floor_index, inventory.gold
            ),
            _ => String::new(),
        };

        /*
         * Fetch and truncate the most recent logbook entries
         */
        let logbook = self.ecs.fetch::<Logbook>();
        let recent_entries = logbook.entries.len().saturating_sub(2);
        let mut serialized_log = String::with_capacity(1024);
        for entry in &logbook.entries[recent_entries..] {
            serialized_log.push_str(entry);
            serialized_log.push('\n');
        }

        // Actually render the split view via ratatui
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(MAX_HEIGHT as u16),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .split(frame.area());
        frame.render_widget(Paragraph::new(Text::from(lines)), layout[0]);
        frame.render_widget(Paragraph::new(Text::from(status_line)), layout[1]);
        frame.render_widget(Paragraph::new(Text::raw(serialized_log)), layout[2]);
    }

    /**
     * Renders the fullscreen logbook, when toggled.
     */
    fn render_log(&self, frame: &mut Frame) {
        let logbook = self.ecs.fetch::<Logbook>();
        let recent_entries = logbook.entries.len().saturating_sub(MAX_HEIGHT as usize);
        let mut serialized_log = String::with_capacity(1024);
        for entry in &logbook.entries[recent_entries..] {
            serialized_log.push_str(entry);
            serialized_log.push('\n');
        }
        frame.render_widget(
            Paragraph::new(Text::raw(serialized_log)).scroll((logbook.scroll_offset, 0)),
            frame.area(),
        );
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
