use std::{io, os::macos::raw::stat};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Padding, Paragraph},
};
use rltk::{Point, RandomNumberGenerator};
use specs::prelude::*;
use specs_derive::Component;
use std::cmp::{max, min};

mod map;
mod map_indexing_system;
mod monster_system;
mod rect;
mod visibility_system;

use crate::{
    map::{MAX_HEIGHT, MAX_WIDTH, Map, TileType, xy_idx},
    map_indexing_system::MapIndexingSystem,
    monster_system::MonsterSystem,
    visibility_system::VisibilitySystem,
};

#[derive(Component, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: char,
    pub fg: Color,
    pub bg: Color,
}

#[derive(Component, Debug)]
pub struct Player {}

#[derive(Component, Debug)]
pub struct Monster {}

#[derive(Component, Debug)]
pub struct Logbook {
    pub entries: Vec<String>,
    pub scroll_offset: u16,
}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    pub range: i32,
}

#[derive(Component)]
pub struct BlocksTile {}

#[derive(Component)]
pub struct Stats {
    pub max_hp: i32,
    pub hp: i32,
    pub strength: i32,
    pub defense: i32,
}

pub struct App {
    pub ecs: World,
    pub dispatcher: Dispatcher<'static, 'static>,
    screen: Screen,
    main_screen: MainScreen,
    menu_index: u8,
    exit: bool,
}

pub enum Screen {
    Menu,
    Main,
}

pub enum MainScreen {
    /**
     * The default view.
     * A split screen between the viewshed and the log.
     */
    Split,

    /**
     * A toggleable view containing a fullscreen logbook.
     */
    Log,
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let stats = ecs.read_storage::<Stats>();
    let mut player_position = ecs.write_resource::<Point>();
    let map = ecs.fetch::<Map>();
    let mut _logbook = ecs.write_resource::<Logbook>();

    for (pos, _player) in (&mut positions, &mut players).join() {
        let next_pos_x = min(MAX_WIDTH - 1, max(0, pos.x + delta_x));
        let next_pos_y = min(MAX_HEIGHT - 1, max(0, pos.y + delta_y));
        let dest = xy_idx(pos.x + delta_x, pos.y + delta_y);

        for entity in map.tile_content[dest].iter() {
            let target = stats.get(*entity);
            match target {
                None => {},
                Some(_t) => {
                    _logbook.entries.push("Attacking!!".to_string());
                    return; 
                }
            }
        }

        let is_blocked_tile = map.blocked_tiles[dest];
        if !is_blocked_tile {
            pos.x = next_pos_x;
            pos.y = next_pos_y;
            player_position.x = next_pos_x;
            player_position.y = next_pos_y;
            // logbook.entries.push(format!("You moved to ({}, {})", pos.x, pos.y));
        }
    }
}

fn try_scroll_logbook(ecs: &mut World, delta: i16) {
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
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        terminal.draw(|frame| self.draw(frame))?;
        while !self.exit {
            self.handle_events()?;
            match self.main_screen {
                MainScreen::Split => self.dispatcher.dispatch(&self.ecs),
                MainScreen::Log => {}
            }
            self.ecs.maintain();
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        match self.screen {
            Screen::Menu => self.render_menu(frame),
            Screen::Main => match self.main_screen {
                MainScreen::Split => self.render_game(frame),
                MainScreen::Log => self.render_log(frame),
            },
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.screen {
            Screen::Menu => match key_event.code {
                KeyCode::Esc => self.exit(),
                KeyCode::Up | KeyCode::Char('w') => {
                    if self.menu_index == 0 {
                        self.menu_index = 1;
                    } else {
                        self.menu_index -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('s') => {
                    if self.menu_index == 1 {
                        self.menu_index = 0;
                    } else {
                        self.menu_index += 1;
                    }
                }
                KeyCode::Enter => match self.menu_index {
                    0 => self.screen = Screen::Main,
                    1 => self.exit(),
                    _ => {}
                },
                _ => {}
            },
            Screen::Main => match self.main_screen {
                MainScreen::Split => match key_event.code {
                    KeyCode::Esc => self.screen = Screen::Menu,

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

                    KeyCode::Char('q') => self.main_screen = MainScreen::Log,
                    KeyCode::Char(' ') => {
                        let mut logbook = self.ecs.fetch_mut::<Logbook>();
                        logbook.entries.push("Letsa go!".to_string());
                    }
                    _ => {}
                },
                MainScreen::Log => match key_event.code {
                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
                        try_scroll_logbook(&mut self.ecs, -1)
                    }

                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
                        try_scroll_logbook(&mut self.ecs, 1)
                    }

                    KeyCode::Char('q') | KeyCode::Esc => self.main_screen = MainScreen::Split,
                    _ => {}
                },
            },
        }
    }

    /**
     * Renders the menu for the game.
     *
     * Should consist of a border and a couple selectable menu items for now.
     * Each one will change the main screen state.
     */
    fn render_menu(&self, frame: &mut Frame) {
        let menu = Block::default()
            .borders(Borders::all())
            .padding(Padding::symmetric(5, 6))
            .inner(frame.area());
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Percentage(50),
                Constraint::Fill(1),
            ])
            .split(menu);
        let horizontal_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Percentage(25),
                Constraint::Fill(1),
            ])
            .split(vertical_layout[1]);
        let menu_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Length(3)])
            .split(horizontal_layout[1]);

        frame.render_widget(
            Paragraph::new(Text::from("New Game"))
                .centered()
                .bg(if self.menu_index == 0 {
                    Color::Cyan
                } else {
                    Color::Black
                })
                .block(Block::bordered().border_set(border::THICK)),
            menu_layout[0],
        );
        frame.render_widget(
            Paragraph::new(Text::from("Quit"))
                .centered()
                .bg(if self.menu_index == 1 {
                    Color::Cyan
                } else {
                    Color::Black
                })
                .block(Block::bordered().border_set(border::THICK)),
            menu_layout[1],
        );
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
         * Overwrite base map spans with any renderable characters
         */
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        for (pos, render) in (&positions, &renderables).join() {
            if map.revealed_tiles[xy_idx(pos.x, pos.y)] {
                lines[pos.y as usize].spans[pos.x as usize] =
                    Span::styled(render.glyph.to_string(), Style::default().fg(render.fg));
            }
        }

        /*
         * Fetch and truncate the most recent logbook entries
         */
        let logbook = self.ecs.fetch::<Logbook>();
        let recent_entries = logbook.entries.len().saturating_sub(3);
        let mut serialized_log = String::with_capacity(1024);
        for entry in &logbook.entries[recent_entries..] {
            serialized_log.push_str(entry);
            serialized_log.push('\n');
        }

        // Actually render the split view via ratatui
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(95), Constraint::Percentage(5)])
            .split(frame.area());
        frame.render_widget(Paragraph::new(Text::from(lines)), layout[0]);
        frame.render_widget(Paragraph::new(Text::raw(serialized_log)), layout[1]);
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

fn main() -> Result<()> {
    color_eyre::install().expect("ahhhh");

    let mut world = World::new();
    world.register::<Position>();
    world.register::<Renderable>();
    world.register::<Player>();
    world.register::<Monster>();
    world.register::<Viewshed>();
    world.register::<BlocksTile>();
    world.register::<Stats>();

    let map = Map::new_map_dynamic_rooms_and_corridors();

    /*
     * Add the player character
     */
    let (player_x, player_y) = map.rooms[0].center();
    world
        .create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: '@',
            bg: Color::Black,
            fg: Color::Yellow,
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
        })
        .with(BlocksTile {})
        .with(Stats {
            max_hp: 20,
            hp: 20,
            strength: 5,
            defense: 2,
        })
        .build();

    /*
     * Add generated monsters
     */
    let mut rng = RandomNumberGenerator::new();
    for room in map.rooms.iter().skip(1) {
        let monster_glyph = match rng.roll_dice(1, 2) {
            1 => 'r',
            2 => 's',
            _ => '?',
        };
        world
            .create_entity()
            .with(Position {
                x: room.center().0,
                y: room.center().1,
            })
            .with(Renderable {
                glyph: monster_glyph,
                bg: Color::Black,
                fg: Color::Red,
            })
            .with(Monster {})
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
            })
            .with(BlocksTile {})
            .with(Stats {
                max_hp: 5,
                hp: 5,
                strength: 1,
                defense: 1,
            })
            .build();
    }

    world.insert(Point::new(player_x, player_y));
    world.insert(map);
    world.insert(Logbook {
        entries: vec!["You begin your adventure in a smallish room...".to_string()],
        scroll_offset: 0,
    });

    let mut dispatcher = DispatcherBuilder::new()
        .with(VisibilitySystem {}, "visibility_system", &[])
        .with(MonsterSystem {}, "monster_system", &[])
        .with(MapIndexingSystem {}, "map_indexing_system", &[])
        .build();
    dispatcher.setup(&mut world);

    let mut terminal = ratatui::init();
    let app_result = App {
        ecs: world,
        dispatcher: dispatcher,
        screen: Screen::Menu,
        main_screen: MainScreen::Split,
        menu_index: 0,
        exit: false,
    }
    .run(&mut terminal);
    ratatui::restore();
    return app_result;
}
