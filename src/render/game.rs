use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Size},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};
use rltk::Point;
use specs::prelude::*;

use crate::{
    RunState, component::{Hidden, Inventory, Item, Name, Pool, Position, Renderable, Stats}, generate::map::{Map, TileType}, logbook::logbook::format_text
};

pub const VIEW_WIDTH: i32 = 80;
pub const VIEW_HEIGHT: i32 = 50;

/**
 * The base render function for the game itself.
 *
 * This should handle rendering the game window itself
 * as well as the log and any other status windows we might need.
 *
 * Game objects themselves should be derived from ecs.
 */
pub fn render_game(ecs: &mut World, frame: &mut Frame, floor_index: u32, _terminal: Size) {
    /*
     * Try to do one large ecs dataset fetch upfront for clarity
     */
    let map = ecs.fetch::<Map>();
    let runstate = ecs.fetch::<RunState>();
    let player_position = ecs.fetch::<Point>();
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let hidden = ecs.read_storage::<Hidden>();
    let player = ecs.fetch::<Entity>();
    let stats = ecs.read_storage::<Stats>();
    let inventory = ecs.read_storage::<Inventory>();
    let names = ecs.read_storage::<Name>();
    let items = ecs.read_storage::<Item>();

    // Define the min (top left), and max (bottom right) of the viewport
    let center = Point {
        x: (VIEW_WIDTH / 2) as i32,
        y: (VIEW_HEIGHT / 2) as i32,
    };
    let map_min = Point {
        x: player_position.x - center.x,
        y: player_position.y - center.y,
    };
    let map_max = Point {
        x: map_min.x + VIEW_WIDTH as i32,
        y: map_min.y + VIEW_HEIGHT as i32,
    };

    /*
     * Create the base map spanlines for the viewport.
     */
    let mut lines: Vec<Line> = Vec::new();
    let mut spans: Vec<Span> = Vec::new();
    for (_view_y, map_y) in (map_min.y ..= map_max.y).enumerate() {
        for (_view_x, map_x) in (map_min.x ..= map_max.x).enumerate() {
            let mut span: Span;

            // Out of bounds on map -- render blanks and avoid any map dereferences
            if map_x < 0 || map_x > map.width - 1 || map_y < 0 || map_y > map.height - 1 {
                span = Span::styled(" ", Style::default());
                spans.push(span);
                continue;
            }

            let map_index = map.xy_idx(map_x, map_y);
            if map.revealed_tiles[map_index] {
                span = match map.tiles[map_index] {
                    TileType::Floor => Span::styled(".", Style::default().fg(Color::Gray)),
                    TileType::Wall => Span::styled("#", Style::default().fg(Color::Green)),
                    TileType::DownStairs => Span::styled("目", Style::default().fg(Color::Yellow))
                }                
            } else {
                span = Span::styled(" ", Style::default());
            }

            if map.bloodstains.contains(&map_index) {
                span = span.bg(Color::Rgb(60, 0, 0));
            }
            spans.push(span);
        }
        lines.push(Line::from(spans));
        spans = Vec::new();
    }

    /*
     * Overwrite base map spans with any renderable characters.
     * Sort renderables by index (render order) prior to rendering, lowest first.
     * 
     * If the existing span has a background set, we keep that (e.g. bloodstain).
     * Otherwise, we use the renderable's desired background.
     */
    let mut renderable_entities = (&positions, &renderables, !&hidden).join().collect::<Vec<_>>();
    renderable_entities.sort_by(|&a, &b| b.1.index.cmp(&a.1.index));
    for (pos, render, _hidden) in renderable_entities.iter() {

        // Renderable has not yet been revealed by the player
        if !map.revealed_tiles[map.xy_idx(pos.x, pos.y)] {
            continue;
        }

        // Renderable is outside of the current viewport
        if pos.x < map_min.x || map_max.x < pos.x || pos.y < map_min.y || map_max.y < pos.y {
            continue;
        }
        let view_pos = Position {
            x: pos.x - map_min.x,
            y: pos.y - map_min.y,
        };

        let existing_span = lines[view_pos.y as usize].spans[view_pos.x as usize].clone();
        lines[view_pos.y as usize].spans[view_pos.x as usize] = Span::styled(
            render.glyph.to_string(),
            Style::default()
                .fg(render.fg)
                .bg(existing_span.style.bg.unwrap_or_else(|| render.bg)),
        );
    }

    /*
     * If the player is in examine mode, overwrite the background of the field
     * being examined with a bright color to indicate that it is selected.
     */
    match *runstate {
        RunState::Examining { index } => {
            let (x, y) = map.idx_xy(index);
            let view_pos = Position {
                x: x - map_min.x,
                y: y - map_min.y,
            };
            let existing_span = lines[view_pos.y as usize].spans[view_pos.x as usize].clone();
            lines[view_pos.y as usize].spans[view_pos.x as usize] = Span::styled(
                existing_span.content,
                Style::default()
                    .fg(existing_span.style.fg.unwrap_or(Color::White))
                    .bg(Color::Cyan)
            );
        },
        _ => {}
    }

    /*
     * Format the status bar with health, gold, etc.
     */
    let player_name: String = names.get(*player).expect("Unable to fetch player name").name.clone();
    let player_floor = format!("Floor: {}", floor_index);
    let pools = format_pools(&player, stats, inventory).expect("Unable to format player pools!");

    /*
     * Fetch and truncate the most recent logbook entries,
     * or the relevant name if in examine mode.
     */
    let text: Text = match *runstate {
        RunState::Examining { index } => {
            let mut serialized_examine: String = "".to_string();
            for entity in map.tile_content.get(index).unwrap_or(&Vec::new()).iter() {
                if let Some(name) = names.get(*entity) {
                    serialized_examine = name.name.clone();
                }

                if let Some(item) = items.get(*entity) {
                    serialized_examine.push('\n');
                    serialized_examine.push_str(&item.description);
                }

                if !serialized_examine.is_empty() { break; }
            }
            Text::raw(serialized_examine)
        },
        _ => {
            format_text(4)
        }
    };

    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Length(VIEW_WIDTH as u16),
            Constraint::Max(40),
        ])
        .split(frame.area());

    let left_block = Block::default().borders(Borders::NONE);
    let right_block = Block::default().borders(Borders::NONE);
    
    frame.render_widget(left_block.clone(), horizontal_layout[0]);
    frame.render_widget(right_block.clone(), horizontal_layout[1]);

    let left_inner = left_block.inner(horizontal_layout[0]);
    let right_inner = right_block.inner(horizontal_layout[1]);

    let left_vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(VIEW_HEIGHT as u16),
            Constraint::Fill(1),
        ])
        .split(left_inner);

    let right_vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(6),
            Constraint::Length(6),
        ])
        .split(right_inner);

    frame.render_widget(Paragraph::new(Text::from(lines)), left_vertical_layout[0]);
    frame.render_widget(
        Paragraph::new(text),
        left_vertical_layout[1]
    );

    frame.render_widget(
        Paragraph::new(
            Text::from(vec![
                Line::from(player_name),
                Line::from(Span::styled(player_floor, Style::new().fg(Color::Gray))),
                Line::from(vec![
                    Span::styled(format!("{:12}", pools.hp.1), Style::new().fg(Color::LightRed)),
                    Span::styled(pools.hp.2, Style::new().bg(Color::Red)),
                    Span::styled(pools.hp.3, Style::new().bg(Color::Rgb(60, 0, 0))),
                ]),
                Line::from(vec![
                    Span::styled(format!("{:12}", pools.mp.1), Style::new().fg(Color::Blue)),
                    Span::styled(pools.mp.2, Style::new().bg(Color::Blue)),
                    Span::styled(pools.mp.3, Style::new().bg(Color::Rgb(0, 0, 60))),
                ]),
                Line::from(vec![
                    Span::styled(format!("{:12}", pools.exp.1), Style::new().fg(Color::LightMagenta)),
                    Span::styled(pools.exp.2, Style::new().bg(Color::LightMagenta)),
                    Span::styled(pools.exp.3, Style::new().bg(Color::Rgb(60, 60, 60))),
                ])
            ]))
            .block(Block::new().borders(Borders::NONE)),
        right_vertical_layout[0]
    );
}

/**
 * The pool itself, followed by formatted strings:
 * - the numeric representation (HP 10 / 30)
 * - the filled bar portion (####)
 * - the unfilled bar portion (____)
 */
pub struct FormattedPools {
    pub hp: (Pool, String, String, String),
    pub mp: (Pool, String, String, String),
    pub exp: (Pool, String, String, String),
}

/*
 * Format the status bar with health, gold, etc.
 */
pub fn format_pools(player: &Entity, stats: ReadStorage<Stats>, inventory: ReadStorage<Inventory>) -> Option<FormattedPools> {
    return match (stats.get(*player), inventory.get(*player)) {
        (Some(stats), Some(_inventory)) => {
            let player_hp = format!("HP: {} / {} ", stats.hp.current, stats.hp.max);
            let hp_bar_remaining = ((stats.hp.current as f64 / stats.hp.max as f64) * (25 as f64)).round() as usize;
            let player_hp_remaining = " ".repeat(hp_bar_remaining);
            let player_hp_total = " ".repeat(25 - hp_bar_remaining);
            
            let player_mp = "MP: 10 / 10 ".to_string();
            let player_mp_remaining = " ".repeat(20);
            let player_mp_total = " ".repeat(5);

            let player_exp = format!("Level: {}", stats.level);
            let player_exp_fill = " ".repeat(
                ((stats.exp.current as f64 / stats.exp.max as f64) * (25 as f64)).round() as usize
            );
            let player_exp_empty = " ".repeat(25 - player_exp_fill.len());

            Some(FormattedPools {
                hp: (stats.hp.clone(), player_hp, player_hp_remaining, player_hp_total),
                mp: (stats.mp.clone(), player_mp, player_mp_remaining, player_mp_total),
                exp: (stats.exp.clone(), player_exp, player_exp_fill, player_exp_empty),
            })
        },
        _ => None,
    }
}
