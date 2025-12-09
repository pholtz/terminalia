use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};
use specs::prelude::*;

use crate::{
    RunState, component::{Hidden, Inventory, Logbook, Name, Position, Renderable, Stats}, generate::map::{MAX_HEIGHT, MAX_WIDTH, Map, TileType, idx_xy, xy_idx}
};

/**
 * The base render function for the game itself.
 *
 * This should handle rendering the game window itself
 * as well as the log and any other status windows we might need.
 *
 * Game objects themselves should be derived from ecs.
 */
pub fn render_game(ecs: &mut World, frame: &mut Frame, floor_index: u32) {
    /*
     * Create the base map lines and spans to render the main game split
     */
    let map = ecs.fetch::<Map>();
    let mut lines = Vec::new();
    let mut spans = Vec::new();
    for (index, tile) in map.tiles.iter().enumerate() {
        let mut span: Span;
        if map.revealed_tiles[index] {
            span = match tile {
                TileType::Floor => Span::styled(".", Style::default().fg(Color::Gray)),
                TileType::Wall => Span::styled("#", Style::default().fg(Color::Green)),
                TileType::DownStairs => Span::styled("目", Style::default().fg(Color::Yellow))
            }
        } else {
            span = Span::styled(" ", Style::default());
        }

        if map.bloodstains.contains(&index) {
            span = span.bg(Color::Rgb(60, 0, 0));
        }
        spans.push(span);

        if (index + 1) % (MAX_WIDTH as usize) == 0 {
            lines.push(Line::from(spans));
            spans = Vec::new();
        }
    }

    /*
     * Overwrite base map spans with any renderable characters.
     * Sort renderables by index (render order) prior to rendering, lowest first.
     * 
     * If the existing span has a background set, we keep that (e.g. bloodstain).
     * Otherwise, we use the renderable's desired background.
     */
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let hidden = ecs.read_storage::<Hidden>();
    let mut renderable_entities = (&positions, &renderables, !&hidden).join().collect::<Vec<_>>();
    renderable_entities.sort_by(|&a, &b| b.1.index.cmp(&a.1.index));
    for (pos, render, _hidden) in renderable_entities.iter() {
        if map.revealed_tiles[xy_idx(pos.x, pos.y)] {
            let existing_span = lines[pos.y as usize].spans[pos.x as usize].clone();
            lines[pos.y as usize].spans[pos.x as usize] = Span::styled(
                render.glyph.to_string(),
                Style::default()
                    .fg(render.fg)
                    .bg(existing_span.style.bg.unwrap_or_else(|| render.bg)),
            );
        }
    }

    /*
     * If the player is in examine mode, overwrite the background of the field
     * being examined with a bright color to indicate that it is selected.
     */
    let runstate = ecs.fetch::<RunState>();
    match *runstate {
        RunState::Examining { index } => {
            let (x, y) = idx_xy(index);
            let existing_span = lines[y as usize].spans[x as usize].clone();
            lines[y as usize].spans[x as usize] = Span::styled(
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
    let player = ecs.fetch::<Entity>();
    let stats = ecs.read_storage::<Stats>();
    let inventory = ecs.read_storage::<Inventory>();
    let runstate = ecs.fetch::<RunState>();
    let names = ecs.read_storage::<Name>();
    let mut player_name: String = "?".to_string();
    let player_floor = format!("Floor: {}", floor_index);
    let mut player_hp: String = "".to_string();
    let mut player_hp_remaining: String = "".to_string();
    let mut player_hp_total: String = "".to_string();
    let mut player_mp: String = "".to_string();
    let mut player_mp_remaining: String = "".to_string();
    let mut player_mp_total: String = "".to_string();
    let status_line = match (stats.get(*player), inventory.get(*player), names.get(*player)) {
        (Some(stats), Some(inventory), Some(name)) => {
            player_name = name.name.clone();
            player_hp = format!("HP: {} / {} ", stats.hp, stats.max_hp);
            let hp_bar_remaining = ((stats.hp as f64 / stats.max_hp as f64) * (25 as f64)).round() as usize;
            player_hp_remaining = " ".repeat(hp_bar_remaining);
            player_hp_total = " ".repeat(25 - hp_bar_remaining);
            player_mp = "MP: 10 / 10 ".to_string();
            player_mp_remaining = " ".repeat(20);
            player_mp_total = " ".repeat(5);
            format!(
                "HP: {} / {}  Floor: {}  Gold: {}  Runstate: {:?}",
                stats.hp, stats.max_hp, floor_index, inventory.gold, *runstate
            )
        },
        _ => String::new(),
    };

    /*
     * Fetch and truncate the most recent logbook entries,
     * or the relevant name if in examine mode.
     */
    let text = match *runstate {
        RunState::Examining { index } => {
            let mut serialized_examine: String = "".to_string();
            for entity in map.tile_content.get(index).unwrap_or(&Vec::new()).iter() {
                if let Some(name) = names.get(*entity) {
                    serialized_examine = name.name.clone();
                    break;
                }
            }
            serialized_examine
        },
        _ => {
            let logbook = ecs.fetch::<Logbook>();
            let recent_entries = logbook.entries.len().saturating_sub(4);
            let mut serialized_log = String::with_capacity(1024);
            for (index, entry) in logbook.entries[recent_entries..].iter().enumerate() {
                serialized_log.push_str(entry);
                if index < logbook.entries[recent_entries..].len() {
                    serialized_log.push('\n');
                }
            }
            serialized_log
        }
    };

    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Length(MAX_WIDTH as u16),
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
            Constraint::Length(MAX_HEIGHT as u16),
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
        Paragraph::new(Text::raw(text)),
        left_vertical_layout[1]
    );

    frame.render_widget(
        Paragraph::new(
            Text::from(vec![
                Line::from(player_name),
                Line::from(Span::styled(player_floor, Style::new().fg(Color::Gray))),
                Line::from(vec![
                    Span::styled(player_hp, Style::new().fg(Color::LightRed)),
                    Span::styled(player_hp_remaining, Style::new().bg(Color::Red)),
                    Span::styled(player_hp_total, Style::new().bg(Color::Rgb(60, 0, 0))),
                ]),
                Line::from(vec![
                    Span::styled(player_mp, Style::new().fg(Color::Blue)),
                    Span::styled(player_mp_remaining, Style::new().bg(Color::Blue)),
                    Span::styled(player_mp_total, Style::new().bg(Color::Rgb(0, 0, 60))),
                ]),
            ]))
            .block(Block::new().borders(Borders::NONE)),
        right_vertical_layout[0]
    );
}
