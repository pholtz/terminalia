use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
};
use specs::prelude::*;

use crate::{
    component::{Inventory, Logbook, Position, Renderable, Stats},
    generate::map::{MAX_HEIGHT, MAX_WIDTH, Map, TileType, xy_idx},
};

/**
 * The base render function for the game itself.
 *
 * This should handle rendering the game window itself
 * as well as the log and any other status windows we might need.
 *
 * Game objects themselves should be derived from ecs.
 */
pub fn render_game(ecs: &mut World, frame: &mut Frame, floor_index: u8) {
    /*
     * Create the base map lines and spans to render the main game split
     */
    let map = ecs.fetch::<Map>();
    let mut lines = Vec::new();
    let mut spans = Vec::new();
    for (index, tile) in map.tiles.iter().enumerate() {
        if map.revealed_tiles[index] {
            match tile {
                TileType::Floor => spans.push(Span::styled(".", Style::default().fg(Color::Gray))),
                TileType::Wall => spans.push(Span::styled("#", Style::default().fg(Color::Green))),
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
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let mut renderable_entities = (&positions, &renderables).join().collect::<Vec<_>>();
    renderable_entities.sort_by(|&a, &b| b.1.index.cmp(&a.1.index));
    for (pos, render) in renderable_entities.iter() {
        if map.revealed_tiles[xy_idx(pos.x, pos.y)] {
            lines[pos.y as usize].spans[pos.x as usize] = Span::styled(
                render.glyph.to_string(),
                Style::default().fg(render.fg).bg(render.bg),
            );
        }
    }

    /*
     * Format the status bar with health, gold, etc.
     */
    let player = ecs.fetch::<Entity>();
    let stats = ecs.read_storage::<Stats>();
    let inventory = ecs.read_storage::<Inventory>();
    let status_line = match (stats.get(*player), inventory.get(*player)) {
        (Some(stats), Some(inventory)) => format!(
            "HP: {} / {}  Floor: {}  Gold: {}",
            stats.hp, stats.max_hp, floor_index, inventory.gold
        ),
        _ => String::new(),
    };

    /*
     * Fetch and truncate the most recent logbook entries
     */
    let logbook = ecs.fetch::<Logbook>();
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
