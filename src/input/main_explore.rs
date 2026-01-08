use crossterm::event::{KeyCode, KeyEvent};
use rltk::Point;
use specs::{prelude::*, storage::GenericWriteStorage};
use std::cmp::{max, min};

use crate::{
    App, RunState, Screen,
    component::{
        Attack, AttackType, EquipmentSlot, Equipped, Item, MagicWeapon, Monster, Player, Pool, Position, RangedWeapon, SpellKnowledge, Stats, WantsToPickupItem
    },
    generate::map::{Map, TileType},
    logbook::logbook::Logger, system::ranged_combat_system::{get_eligible_ranged_tiles, has_line_of_sight},
};

pub fn handle_main_explore_key_event(
    app: &mut App,
    runstate: RunState,
    key_event: KeyEvent,
) -> Option<RunState> {
    match key_event.code {
        KeyCode::Esc => match runstate {
            RunState::Examining { index: _ } => Some(RunState::AwaitingInput),
            RunState::FreeAiming { index: _ } => Some(RunState::AwaitingInput),
            RunState::AwaitingInput => {
                app.screen = Screen::Quit { quit: false };
                None
            }
            _ => None,
        },

        KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('h') => match runstate {
            RunState::AwaitingInput => try_move_player(-1, 0, &mut app.ecs),
            RunState::Examining { index: _ } => try_move_examine(app, -1, 0),
            RunState::FreeAiming { index: _ } => try_move_free_aim(app, -1, 0),
            _ => None,
        },

        KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('l') => match runstate {
            RunState::AwaitingInput => try_move_player(1, 0, &mut app.ecs),
            RunState::Examining { index: _ } => try_move_examine(app, 1, 0),
            RunState::FreeAiming { index: _ } => try_move_free_aim(app, 1, 0),
            _ => None,
        },

        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => match runstate {
            RunState::AwaitingInput => try_move_player(0, -1, &mut app.ecs),
            RunState::Examining { index: _ } => try_move_examine(app, 0, -1),
            RunState::FreeAiming { index: _ } => try_move_free_aim(app, 0, -1),
            _ => None,
        },

        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => match runstate {
            RunState::AwaitingInput => try_move_player(0, 1, &mut app.ecs),
            RunState::Examining { index: _ } => try_move_examine(app, 0, 1),
            RunState::FreeAiming { index: _ } => try_move_free_aim(app, 0, 1),
            _ => None,
        },

        KeyCode::Char('/') => {
            let ecs = &mut app.ecs;
            let map = ecs.fetch::<Map>();
            let player = ecs.read_resource::<Entity>();
            let positions = ecs.read_storage::<Position>();
            let position = positions
                .get(*player)
                .expect("Cannot get position for player");
            return match app.runstate {
                RunState::Examining { index: _ } => Some(RunState::AwaitingInput),
                _ => Some(RunState::Examining {
                    index: map.xy_idx(position.x, position.y),
                }),
            };
        }

        KeyCode::Tab => try_cycle_targeting(&mut app.ecs),
        KeyCode::Char(' ') => try_free_aim(app),
        KeyCode::Char('1') => try_ranged_target(app),
        KeyCode::Char('2') => try_magic_target(app),
        KeyCode::Char('g') => try_get_item(&mut app.ecs),
        KeyCode::Char('i') => {
            app.screen = Screen::Inventory;
            return None;
        }
        KeyCode::Char('.') => try_next_level(&mut app.ecs),
        KeyCode::Char(',') => try_prev_level(&mut app.ecs),

        /*
         * Cheats
         */
        KeyCode::Char('0') => {
            let ecs = &mut app.ecs;
            let player = ecs.fetch::<Entity>();
            let mut stats = ecs.write_storage::<Stats>();
            let player_stats = stats
                .get_mut(*player)
                .expect("Unable to access player stats");
            player_stats.hp = Pool {
                current: player_stats.hp.max,
                max: player_stats.hp.max,
            };
            return None;
        }
        KeyCode::Char('q') => {
            app.screen = Screen::Log;
            return None;
        }
        _ => None,
    }
}

fn try_move_examine(app: &mut App, delta_x: i32, delta_y: i32) -> Option<RunState> {
    match app.runstate {
        RunState::Examining { index } => {
            let map = app.ecs.fetch::<Map>();
            let (x, y) = map.idx_xy(index);
            let next_x = min(map.width - 1, max(0, x + delta_x));
            let next_y = min(map.height - 1, max(0, y + delta_y));
            return Some(RunState::Examining {
                index: map.xy_idx(next_x, next_y),
            });
        }
        _ => None,
    }
}

fn try_move_free_aim(app: &mut App, delta_x: i32, delta_y: i32) -> Option<RunState> {
    match app.runstate {
        RunState::FreeAiming { index } => {
            let map = app.ecs.fetch::<Map>();
            let player_pos = app.ecs.fetch::<Point>();
            let player_entity = app.ecs.fetch::<Entity>();
            let entities = app.ecs.entities();
            let equipped = app.ecs.read_storage::<Equipped>();
            let ranged_weapons = app.ecs.read_storage::<RangedWeapon>();
            let magic_weapons = app.ecs.read_storage::<MagicWeapon>();
            let ranged_mask = ranged_weapons.mask() | magic_weapons.mask();

            for (entity, equipped, _) in (&entities, &equipped, &ranged_mask).join() {
                let ranged = ranged_weapons.get(entity);
                let magic = magic_weapons.get(entity);
                let range = ranged.map(|r| r.range).unwrap_or_else(|| magic.map(|m| m.range).unwrap_or(0));
                if equipped.slot == EquipmentSlot::Weapon && equipped.owner == *player_entity {
                    let (x, y) = map.idx_xy(index);
                    let target_index = map.xy_idx(x + delta_x, y + delta_y);
                    let eligible_tiles = get_eligible_ranged_tiles(&map, &player_pos, range);
                    if eligible_tiles.contains(&target_index) {
                        return Some(RunState::FreeAiming { index: target_index });
                    }
                }
            }
            return None;
        },
        _ => None,
    }
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) -> Option<RunState> {
    let entities = ecs.entities();
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut attacks = ecs.write_storage::<Attack>();
    let stats = ecs.read_storage::<Stats>();
    let mut player_position = ecs.write_resource::<Point>();
    let map = ecs.fetch::<Map>();

    for (entity, pos, _player) in (&entities, &mut positions, &mut players).join() {
        let next_pos_x = min(map.width - 1, max(0, pos.x + delta_x));
        let next_pos_y = min(map.height - 1, max(0, pos.y + delta_y));
        let dest = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        for target in map.tile_content[dest].iter() {
            let target_stats = stats.get(*target);
            match target_stats {
                None => {}
                Some(_t) => {
                    attacks
                        .insert(
                            entity,
                            Attack {
                                attack_type: AttackType::Melee,
                                target: *target,
                                spell: None,
                            },
                        )
                        .expect("Unable to add attack");
                    return Some(RunState::PlayerTurn);
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
    return Some(RunState::PlayerTurn);
}

fn try_get_item(ecs: &mut World) -> Option<RunState> {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => Logger::new()
            .append("There is nothing here to pick up.")
            .log(),
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
    return Some(RunState::PlayerTurn);
}

fn try_next_level(ecs: &mut World) -> Option<RunState> {
    let mut runstate = ecs.write_resource::<RunState>();
    let map = ecs.read_resource::<Map>();
    let player_position = ecs.read_resource::<Point>();
    let player_index = map.xy_idx(player_position.x, player_position.y);
    if map.tiles[player_index] == TileType::DownStairs {
        *runstate = RunState::Descending;
        return Some(RunState::Descending);
    }
    return None;
}

fn try_prev_level(ecs: &mut World) -> Option<RunState> {
    let mut runstate = ecs.write_resource::<RunState>();
    let map = ecs.read_resource::<Map>();
    let player_position = ecs.read_resource::<Point>();
    let player_index = map.xy_idx(player_position.x, player_position.y);
    if map.tiles[player_index] == TileType::UpStairs {
        *runstate = RunState::Ascending;
        return Some(RunState::Ascending);
    }
    return None;
}

fn try_cycle_targeting(ecs: &mut World) -> Option<RunState> {
    let entities = ecs.entities();
    let map = ecs.fetch::<Map>();
    let player_entity = ecs.fetch::<Entity>();
    let equipped = ecs.read_storage::<Equipped>();
    let mut ranged_weapons = ecs.write_storage::<RangedWeapon>();
    let mut magic_weapons = ecs.write_storage::<MagicWeapon>();
    let monsters = ecs.read_storage::<Monster>();
    let positions = ecs.read_storage::<Position>();

    let mut player_ranged_weapon: Option<&mut RangedWeapon> = None;
    for (_entity, equipped, ranged_weapon) in (&entities, &equipped, &mut ranged_weapons).join() {
        if equipped.slot == EquipmentSlot::Weapon && equipped.owner == *player_entity {
            player_ranged_weapon = Some(ranged_weapon);
        }
    }

    match player_ranged_weapon {
        Some(ranged) => {
            let player_pos = positions
                .get(*player_entity)
                .expect("Unable to access player position");

            let mut eligible_monsters = Vec::new();
            for (monster_entity, _monster, monster_pos) in (&entities, &monsters, &positions).join() {
                let player = Point { x: player_pos.x, y: player_pos.y };
                let monster = Point { x: monster_pos.x, y: monster_pos.y };
                let distance = rltk::DistanceAlg::Pythagoras.distance2d(player, monster);
                if distance <= ranged.range as f32 && has_line_of_sight(&map, player, monster) {
                    eligible_monsters
                        .push((map.xy_idx(monster_pos.x, monster_pos.y), monster_entity));
                }
            }

            eligible_monsters.sort_by_key(|(idx, _)| *idx);
            if !eligible_monsters.is_empty() {
                match ranged.target {
                    Some(target) => {
                        let existing_target = eligible_monsters
                            .iter()
                            .enumerate()
                            .filter(|(_index, (_map_index, monster))| *monster == target)
                            .next();
                        match existing_target {
                            Some(et) => {
                                let next_index = et.0 + 1;
                                if next_index < eligible_monsters.len() {
                                    ranged.target = Some(eligible_monsters[next_index].1);
                                } else {
                                    ranged.target = Some(eligible_monsters[0].1);
                                }
                            }
                            None => {
                                ranged.target = Some(eligible_monsters[0].1);
                            }
                        }
                    }
                    None => {
                        ranged.target = Some(eligible_monsters[0].1);
                    }
                }
            }
        }
        None => {}
    }

    let mut player_magic_weapon: Option<&mut MagicWeapon> = None;
    for (_entity, equipped, magic_weapon) in (&entities, &equipped, &mut magic_weapons).join() {
        if equipped.slot == EquipmentSlot::Weapon && equipped.owner == *player_entity {
            player_magic_weapon = Some(magic_weapon);
        }
    }

    match player_magic_weapon {
        Some(magic) => {
            let player_pos = positions
                .get(*player_entity)
                .expect("Unable to access player position");

            let mut eligible_monsters = Vec::new();
            for (monster_entity, _monster, monster_pos) in (&entities, &monsters, &positions).join()
            {
                let player = Point { x: player_pos.x, y: player_pos.y };
                let monster = Point { x: monster_pos.x, y: monster_pos.y };
                let distance = rltk::DistanceAlg::Pythagoras.distance2d(player, monster);
                if distance <= magic.range as f32 && has_line_of_sight(&map, player, monster) {
                    eligible_monsters
                        .push((map.xy_idx(monster_pos.x, monster_pos.y), monster_entity));
                }
            }

            eligible_monsters.sort_by_key(|(idx, _)| *idx);
            if !eligible_monsters.is_empty() {
                match magic.target {
                    Some(target) => {
                        let existing_target = eligible_monsters
                            .iter()
                            .enumerate()
                            .filter(|(_index, (_map_index, monster))| *monster == target)
                            .next();
                        match existing_target {
                            Some(et) => {
                                let next_index = et.0 + 1;
                                if next_index < eligible_monsters.len() {
                                    magic.target = Some(eligible_monsters[next_index].1);
                                } else {
                                    magic.target = Some(eligible_monsters[0].1);
                                }
                            }
                            None => {
                                magic.target = Some(eligible_monsters[0].1);
                            }
                        }
                    }
                    None => {
                        magic.target = Some(eligible_monsters[0].1);
                    }
                }
            }
        }
        None => {}
    }

    return None;
}

fn try_free_aim(app: &mut App) -> Option<RunState> {
    let map = app.ecs.fetch::<Map>();
    let player_pos = app.ecs.fetch::<Point>();
    let player_entity = app.ecs.fetch::<Entity>();
    let equipped = app.ecs.read_storage::<Equipped>();
    let ranged_weapons = app.ecs.read_storage::<RangedWeapon>();
    let magic_weapons = app.ecs.read_storage::<MagicWeapon>();
    let ranged_mask = ranged_weapons.mask() | magic_weapons.mask();

    // Ranged or magic weapon required for switching to free aim
    for (equipped, _) in (&equipped, &ranged_mask).join() {
        if equipped.slot == EquipmentSlot::Weapon && equipped.owner == *player_entity {
            match app.runstate {
                RunState::FreeAiming { index: _ } => return Some(RunState::AwaitingInput),
                _ => return Some(RunState::FreeAiming { index: map.xy_idx(player_pos.x, player_pos.y) }),
            }
        }
    }
    return None;
}

/// Attacks the currently selected ranged target with the currently equipped
/// ranged or magic weapon, if possible.
/// 
/// Handles both freeaim and targeting scenarios.
/// 
fn try_ranged_target(app: &mut App) -> Option<RunState> {
    let entities = app.ecs.entities();
    let map = app.ecs.fetch::<Map>();
    let player_entity = app.ecs.fetch::<Entity>();
    let equipped = app.ecs.read_storage::<Equipped>();
    let spell_knowledge = app.ecs.read_storage::<SpellKnowledge>();
    let mut ranged_weapons = app.ecs.write_storage::<RangedWeapon>();
    let mut magic_weapons = app.ecs.write_storage::<MagicWeapon>();
    let mut attacks = app.ecs.write_storage::<Attack>();
    let ranged_mask = ranged_weapons.mask().clone() | magic_weapons.mask().clone();

    for (entity, equipped, _) in
        (&entities, &equipped, &ranged_mask).join()
    {
        if equipped.slot != EquipmentSlot::Weapon || equipped.owner != *player_entity {
            continue;
        }
        let ranged_weapon = ranged_weapons.get_mut(entity);
        let magic_weapon = magic_weapons.get_mut(entity);
        let attack_type = if ranged_weapon.is_some() { AttackType::Ranged } else { AttackType::Magic };
        let target = ranged_weapon.map(|r| r.target)
            .unwrap_or_else(|| magic_weapon.map(|m| m.target).unwrap_or(None));

        let spell = match attack_type {
            AttackType::Magic => {
                spell_knowledge.get(*player_entity)
                    .expect("uhhh")
                    .spells
                    .first()
                    .map(|s| s.clone())
            }
            _ => None
        };

        match app.runstate {                
            RunState::FreeAiming { index } => {
                match map.tile_content[index].iter().next() {
                    Some(entity) => {
                        attacks.insert(
                            *player_entity,
                            Attack {
                                attack_type: attack_type,
                                target: *entity,
                                spell: spell,
                            }
                        ).expect("Unable to add attack");
                        return Some(RunState::PlayerTurn);
                    },
                    None => return None,
                }
            }
            _ => match target {
                Some(target) => {
                    attacks
                        .insert(
                            *player_entity,
                            Attack {
                                attack_type: attack_type,
                                target: target,
                                spell: spell,
                            },
                        )
                        .expect("Unable to add attack");
                    return Some(RunState::PlayerTurn);
                },
                None => return None,
            }
        }
    }
    return None;
}

fn try_magic_target(app: &mut App) -> Option<RunState> {
    let map = app.ecs.fetch::<Map>();
    let player_entity = app.ecs.fetch::<Entity>();
    let spell_knowledge = app.ecs.read_storage::<SpellKnowledge>();
    let mut attacks = app.ecs.write_storage::<Attack>();
    if let Some(spells) = spell_knowledge.get(*player_entity) {
        let spell = spells.spells.first().expect("uhh");
        match app.runstate {
            RunState::FreeAiming { index } => {
                match map.tile_content[index].iter().next() {
                    Some(entity) => {
                        attacks.insert(
                            *player_entity,
                            Attack {
                                attack_type: AttackType::Magic,
                                target: *entity,
                                spell: Some(spell.clone()),
                            }
                        ).expect("Unable to add magic attack");
                        return Some(RunState::PlayerTurn);
                    },
                    _ => return None
                }
            }
            _ => return None
        }
        // TODO: Make targeting it's own component, not scoped to a weapon
    }
    return None;
}
