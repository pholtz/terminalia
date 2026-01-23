use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Size},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap},
};
use rltk::Point;
use specs::prelude::*;

use crate::{
    RunState, component::{
        EquipmentSlot, Equipped, Hidden, Inventory, Item, MagicWeapon, Name, Npc, Pool, Position, RangedWeapon, Renderable, Stats
    }, generate::map::{Map, TileType}, logbook::logbook::format_latest_text, render::base::centered_rect, system::ranged_combat_system::get_eligible_ranged_tiles
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
    let player_entity = ecs.fetch::<Entity>();
    let player_position = ecs.fetch::<Point>();
    let entities = ecs.entities();
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let hidden = ecs.read_storage::<Hidden>();
    let player = ecs.fetch::<Entity>();
    let stats = ecs.read_storage::<Stats>();
    let inventory = ecs.read_storage::<Inventory>();
    let names = ecs.read_storage::<Name>();
    let items = ecs.read_storage::<Item>();
    let ranged_weapons = ecs.read_storage::<RangedWeapon>();
    let magic_weapons = ecs.read_storage::<MagicWeapon>();
    let equipped = ecs.read_storage::<Equipped>();

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
    for (_view_y, map_y) in (map_min.y..=map_max.y).enumerate() {
        for (_view_x, map_x) in (map_min.x..=map_max.x).enumerate() {
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
                    TileType::DownStairs => Span::styled(">", Style::default().fg(Color::Yellow)),
                    TileType::UpStairs => Span::styled("<", Style::default().fg(Color::Yellow)),
                    TileType::Debris => Span::styled("◯", Style::default().fg(Color::White)),
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
    let mut renderable_entities = (&positions, &renderables, !&hidden)
        .join()
        .collect::<Vec<_>>();
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

    // Create a bitmask to allow us to union (OR) ranged and magic weapons
    let mut ranged_mask = BitSet::new();
    ranged_mask |= ranged_weapons.mask();
    ranged_mask |= magic_weapons.mask();

    /*
     * E X A M I N I N G
     * If the player is in examine mode, overwrite the background of the field
     * being examined with a bright color to indicate that it is selected.
     *
     * F R E E  A I M I N G
     * If the player is free aiming, we want to retrieve the entity they are
     * aiming with and brightly render all possible tiles within range.
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
                    .bg(Color::Cyan),
            );
        }
        RunState::FreeAiming { index } => {
            for (entity, equipped, _) in (&entities, &equipped, &ranged_mask).join() {
                let ranged = ranged_weapons.get(entity);
                let magic = magic_weapons.get(entity);
                let range = ranged.map(|r| r.range).unwrap_or_else(|| magic.map(|m| m.range).unwrap_or(0));
                if equipped.slot == EquipmentSlot::Weapon && equipped.owner == *player_entity {
                    let eligible_tiles =
                        get_eligible_ranged_tiles(&map, &player_position, range);
                    for tile_index in eligible_tiles.iter() {
                        let (tile_x, tile_y) = map.idx_xy(*tile_index);
                        let view_pos = Position {
                            x: tile_x - map_min.x,
                            y: tile_y - map_min.y,
                        };
                        let existing_span =
                            lines[view_pos.y as usize].spans[view_pos.x as usize].clone();
                        lines[view_pos.y as usize].spans[view_pos.x as usize] = Span::styled(
                            existing_span.content,
                            Style::default()
                                .fg(existing_span.style.fg.unwrap_or(Color::White))
                                .bg(if index == *tile_index {
                                    Color::Red
                                } else {
                                    Color::LightGreen
                                }),
                        );
                    }
                }
            }
        }
        _ => {}
    }

    /*
     * Targeting
     * If the player is targeting an enemy, we should overwrite the background
     * of the entity with a bright color to indicate that it is targeted.
     */
    for (entity, equipped, _) in (&entities, &equipped, &ranged_mask).join() {
        let ranged = ranged_weapons.get(entity);
        let magic = magic_weapons.get(entity);
        let target = ranged.map(|r| r.target).unwrap_or_else(|| magic.map(|m| m.target).unwrap_or(None));
        let is_ranged = equipped.slot == EquipmentSlot::Weapon && equipped.owner == *player;
        let is_targeting = target.is_some();
        if is_ranged && is_targeting {
            if let Some(target_pos) = positions.get(target.unwrap()) {
                // Renderable is outside of the current viewport
                if target_pos.x < map_min.x
                    || map_max.x < target_pos.x
                    || target_pos.y < map_min.y
                    || map_max.y < target_pos.y
                {
                    continue;
                }
                let view_pos = Position {
                    x: target_pos.x - map_min.x,
                    y: target_pos.y - map_min.y,
                };
                let existing_span = lines[view_pos.y as usize].spans[view_pos.x as usize].clone();
                lines[view_pos.y as usize].spans[view_pos.x as usize] = Span::styled(
                    existing_span.content,
                    Style::default()
                        .fg(existing_span.style.fg.unwrap_or(Color::White))
                        .bg(Color::LightGreen),
                );
            }
        }
    }

    /*
     * Format the status bar with health, gold, etc.
     */
    let player_name: String = names
        .get(*player)
        .expect("Unable to fetch player name")
        .name
        .clone();
    let player_floor = format!("Floor: {}", floor_index);
    let pools = format_pools(&player, stats, inventory).expect("Unable to format player pools!");

    /*
     * Fetch and truncate the most recent logbook entries,
     * or the relevant name if in examine mode.
     */
    let text: Text = match *runstate {
        RunState::Examining { index } => {
            if *map.revealed_tiles.get(index).unwrap_or(&false) {
                let mut serialized_examine: String = "".to_string();
                for entity in map.tile_content.get(index).unwrap_or(&Vec::new()).iter() {
                    let name = names.get(*entity);
                    let item = items.get(*entity);

                    if let Some(name) = name {
                        if !serialized_examine.is_empty() {
                            serialized_examine.push('\n');
                        }
                        serialized_examine.push_str(&name.name);
                    }

                    if let Some(item) = item {
                        if !serialized_examine.is_empty() {
                            serialized_examine.push('\n');
                        }
                        serialized_examine.push_str(&item.description);
                    }

                    if name.is_some() || item.is_some() {
                        break; // only examine the first entity
                    }
                }

                // Always include TileType details below entity details
                if let Some(tile_type) = map.tiles.get(index) {
                    if !serialized_examine.is_empty() {
                        serialized_examine.push('\n');
                    }
                    serialized_examine.push_str(&format!(
                        "Tile: {:?} -> {}",
                        tile_type,
                        tile_type.description()
                    ));
                }
                Text::raw(serialized_examine)
            } else {
                Text::from("???")
            }
        }
        _ => format_latest_text(4),
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
        .constraints(vec![Constraint::Length(6), Constraint::Length(6)])
        .split(right_inner);

    frame.render_widget(Paragraph::new(Text::from(lines)), left_vertical_layout[0]);
    frame.render_widget(Paragraph::new(text), left_vertical_layout[1]);

    frame.render_widget(
        Paragraph::new(Text::from(vec![
            Line::from(player_name),
            Line::from(Span::styled(player_floor, Style::new().fg(Color::Gray))),
            Line::from(vec![
                Span::styled(
                    format!("{:12}", pools.hp.1),
                    Style::new().fg(Color::LightRed),
                ),
                Span::styled(pools.hp.2, Style::new().bg(Color::Red)),
                Span::styled(pools.hp.3, Style::new().bg(Color::Rgb(60, 0, 0))),
            ]),
            Line::from(vec![
                Span::styled(
                    format!("{:12}", pools.mp.1),
                    Style::new().fg(Color::Blue)
                ),
                Span::styled(pools.mp.2, Style::new().bg(Color::Blue)),
                Span::styled(pools.mp.3, Style::new().bg(Color::Rgb(0, 0, 60))),
            ]),
            Line::from(vec![
                Span::styled(
                    format!("{:12}", pools.exp.1),
                    Style::new().fg(Color::LightMagenta),
                ),
                Span::styled(pools.exp.2, Style::new().bg(Color::LightMagenta)),
                Span::styled(pools.exp.3, Style::new().bg(Color::Rgb(60, 60, 60))),
            ]),
        ]))
        .block(Block::new().borders(Borders::NONE)),
        right_vertical_layout[0],
    );

    /*
     * D I A L O G U E  M O D A L
     * 
     * In the event of npc dialogue, we want to render a part screen modal over
     * the explore window. We do this by calculating a subset of the frame area
     * and rendering a paragraph over it last, to effectively overwrite the output.
     */
    match *runstate {
        RunState::Dialogue { npc } => {
            let npcs = ecs.read_storage::<Npc>();
            let dialogue = npcs.get(npc).expect("Unable to access given npc component").dialogue.clone();
            if dialogue.is_some() {
                let modal_area = centered_rect(90, 90, left_vertical_layout[0]);
                frame.render_widget(Clear, modal_area);
                frame.render_widget(
                    Paragraph::new(Text::from(dialogue.unwrap().join("\n\n")))
                        .style(Style::default().fg(Color::White).bg(Color::Black))
                        .wrap(Wrap { trim: true })
                        .block(Block::default()
                            .title("Dialogue")
                            .title_alignment(Alignment::Center)
                            .borders(Borders::ALL)
                            .padding(Padding::uniform(2))
                        ),
                    modal_area,
                );
            }
        }
        _ => {}
    }
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
pub fn format_pools(
    player: &Entity,
    stats: ReadStorage<Stats>,
    inventory: ReadStorage<Inventory>,
) -> Option<FormattedPools> {
    return match (stats.get(*player), inventory.get(*player)) {
        (Some(stats), Some(_inventory)) => {
            let player_hp = format!("HP: {} / {} ", stats.hp.current, stats.hp.max);
            let hp_bar_remaining =
                ((stats.hp.current as f64 / stats.hp.max as f64) * (25 as f64)).round() as usize;
            let player_hp_remaining = " ".repeat(hp_bar_remaining);
            let player_hp_total = " ".repeat(25 - hp_bar_remaining);

            let player_mp = format!("MP: {} / {} ", stats.mp.current, stats.mp.max);
            let mp_bar_remaining =
                ((stats.mp.current as f64 / stats.mp.max as f64) * (25 as f64)).round() as usize;
            let player_mp_remaining = " ".repeat(mp_bar_remaining);
            let player_mp_total = " ".repeat(25 - mp_bar_remaining);

            let player_exp = format!("Level: {}", stats.level);
            let player_exp_fill = " ".repeat(
                ((stats.exp.current as f64 / stats.exp.max as f64) * (25 as f64)).round() as usize,
            );
            let player_exp_empty = " ".repeat(25 - player_exp_fill.len());

            Some(FormattedPools {
                hp: (
                    stats.hp.clone(),
                    player_hp,
                    player_hp_remaining,
                    player_hp_total,
                ),
                mp: (
                    stats.mp.clone(),
                    player_mp,
                    player_mp_remaining,
                    player_mp_total,
                ),
                exp: (
                    stats.exp.clone(),
                    player_exp,
                    player_exp_fill,
                    player_exp_empty,
                ),
            })
        }
        _ => None,
    };
}
