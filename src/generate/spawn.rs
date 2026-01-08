use std::{fs, sync::Mutex};

use indexmap::IndexMap;
use lazy_static::lazy_static;
use ratatui::style::Color;
use rltk::RandomNumberGenerator;
use specs::prelude::*;

use crate::{
    component::{
        Armor, BlocksTile, Equippable, Hidden, Inventory, Item, MagicMapper, MagicWeapon, MeleeWeapon, Monster, Name, Player, Pool, Position, Potion, RangedWeapon, Renderable, Spell, SpellKnowledge, Stats, Triggerable, Viewshed
    },
    generate::{config::{DropConfig, DropType, ItemConfig, MonsterConfig, ScrollType, parse_dice_expression}, random_table::RandomTable, rect::Rect},
};

lazy_static! {
    pub static ref ITEMS: Mutex<Vec<ItemConfig>> = Mutex::new(Vec::new());
    pub static ref MONSTERS: Mutex<Vec<MonsterConfig>> = Mutex::new(Vec::new());
    pub static ref DROPS: Mutex<Vec<DropConfig>> = Mutex::new(Vec::new());
}

pub fn initialize_config() {
    let items_raw = fs::read_to_string("./config/items.json").unwrap();
    let items: Vec<ItemConfig> = serde_json::from_str(&items_raw).unwrap();
    ITEMS.lock().unwrap().extend(items);

    let monsters_raw = fs::read_to_string("./config/monsters.json").unwrap();
    let monsters: Vec<MonsterConfig> = serde_json::from_str(&monsters_raw).unwrap();
    MONSTERS.lock().unwrap().extend(monsters);

    let drops_raw = fs::read_to_string("./config/drops.json").unwrap();
    let drops: Vec<DropConfig> = serde_json::from_str(&drops_raw).unwrap();
    DROPS.lock().unwrap().extend(drops);
}

/// Spawns a weighted item based on the current floor and an internal spawn table.
pub fn spawn_weighted_item(ecs: &mut World, floor_index: u32, room: &Rect) {
    let (pos, spawn): (Position, String) = {
        let mut rng = ecs.fetch_mut::<RandomNumberGenerator>();
        let width = room.x2 - room.x1;
        let height = room.y2 - room.y1;
        let x = room.x1 + rng.roll_dice(1, width - 1);
        let y = room.y1 + rng.roll_dice(1, height - 1);
        let pos = Position { x: x, y: y };

        let mut item_spawn_table = RandomTable::new();
        for item in ITEMS.lock().unwrap().iter() {
            match &item.spawn {
                Some(spawn) => {
                    item_spawn_table.push(item.name.clone(), spawn.base_weight);
                }
                None => {}
            };
        }
        (pos, item_spawn_table.roll(&mut rng))
    };

    for item in ITEMS.lock().unwrap().iter() {
        if item.name != spawn {
            continue;
        }
        let mut entity = ecs.create_entity();
        entity = spawn_item(entity, pos, item);
        entity.build();
        break;
    }
}

/// Spawns a weighted monster based on the current floor and internal spawn table.
pub fn spawn_weighted_monster(ecs: &mut World, floor_index: u32, room: &Rect) {
    let (pos, spawn): (Position, String) = {
        let mut rng = ecs.fetch_mut::<RandomNumberGenerator>();
        let width = room.x2 - room.x1;
        let height = room.y2 - room.y1;
        let x = room.x1 + rng.roll_dice(1, width - 1);
        let y = room.y1 + rng.roll_dice(1, height - 1);
        let pos = Position { x: x, y: y };

        let mut monster_spawn_table = RandomTable::new();
        for monster in MONSTERS.lock().unwrap().iter() {
            match &monster.spawn {
                Some(spawn) => {
                    monster_spawn_table.push(monster.name.clone(), spawn.base_weight);
                }
                None => {}
            };
        }
        (pos, monster_spawn_table.roll(&mut rng))
    };

    for monster in MONSTERS.lock().unwrap().iter() {
        if monster.name != spawn { continue; }
        let mut entity = ecs
            .create_entity()
            .with(pos)
            .with(Name {
                name: monster.name.clone(),
            })
            .with(Monster {
                drop_type: monster.drop_type.clone(),
            });
        
        match &monster.renderable {
            Some(renderable) => {
                entity = entity.with(Renderable {
                    glyph: renderable.glyph.chars().next().unwrap_or('!'),
                    fg: renderable
                        .fg
                        .clone()
                        .map(|fg| color_from_hex(fg.as_str()).unwrap())
                        .unwrap_or(Color::default()),
                    bg: renderable
                        .bg
                        .clone()
                        .map(|bg| color_from_hex(bg.as_str()).unwrap())
                        .unwrap_or(Color::default()),
                    index: renderable.index,
                })
            },
            None => {},
        }

        match &monster.viewshed {
            Some(viewshed) => {
                entity = entity.with(Viewshed {
                    range: viewshed.range,
                    visible_tiles: Vec::new(),
                });
            },
            None => {},
        }

        match &monster.stats {
            Some(stats) => {
                entity = entity.with(Stats {
                    hp: Pool { current: stats.hp.current, max: stats.hp.max },
                    mp: Pool { current: stats.mp.current, max: stats.mp.max },
                    exp: Pool { current: stats.exp.current, max: stats.exp.max },
                    level: stats.level,
                    strength: stats.strength,
                    dexterity: stats.dexterity,
                    constitution: stats.constitution,
                    intelligence: stats.intelligence,
                    wisdom: stats.wisdom,
                    charisma: stats.charisma,
                });
            },
            None => {},
        }

        entity.build();
        break;
    }
}

/// Spawns one or more items using a supporting drop table.
/// Intended to be used to spawn items when monsters are defeated.
pub fn spawn_weighted_drop(ecs: &mut World, drop_type: DropType, pos: Position) {
    let mut rng = ecs.fetch_mut::<RandomNumberGenerator>();
    let mut drop_spawn_table = RandomTable::new();
    if let Some(drop) = DROPS.lock().unwrap().iter().find(|drop| drop.drop_type == drop_type) {
        for drop_choice in drop.drops.iter() {
            drop_spawn_table.push(drop_choice.name.clone(), drop_choice.weight);
        }
        let selected_item = drop_spawn_table.roll(&mut rng);
        if let Some(item) = ITEMS.lock().unwrap().iter().find(|item| item.name == selected_item) {
            let mut entity = ecs.create_entity_unchecked();
            entity = spawn_item(entity, pos, item);
            entity.build();
        }
    }
}

pub fn spawn_item<'a>(mut entity: EntityBuilder<'a>, pos: Position, item: &ItemConfig) -> EntityBuilder<'a> {
    entity = entity.with(pos)
        .with(Name {
            name: item.name.clone(),
        })
        .with(Item {
            description: item.description.clone(),
        });

    match &item.renderable {
        Some(renderable) => {
            entity = entity.with(Renderable {
                glyph: renderable.glyph.chars().next().unwrap_or('!'),
                fg: renderable
                    .fg
                    .clone()
                    .map(|fg| color_from_hex(fg.as_str()).unwrap())
                    .unwrap_or(Color::default()),
                bg: renderable
                    .bg
                    .clone()
                    .map(|bg| color_from_hex(bg.as_str()).unwrap())
                    .unwrap_or(Color::default()),
                index: renderable.index,
            });
        }
        None => {}
    }

    match &item.potion {
        Some(potion) => {
            entity = entity.with(Potion {
                heal_amount: potion.heal_amount,
            });
        }
        None => {}
    }

    match &item.equippable {
        Some(equippable) => {
            entity = entity.with(Equippable {
                slot: equippable.slot,
            });
        }
        None => {}
    }

    match &item.melee_weapon {
        Some(melee_weapon) => {
            entity = entity.with(MeleeWeapon {
                damage: parse_dice_expression(&melee_weapon.damage),
            });
        }
        None => {}
    }

    match &item.ranged_weapon {
        Some(ranged_weapon) => {
            entity = entity.with(RangedWeapon {
                damage: parse_dice_expression(&ranged_weapon.damage),
                range: ranged_weapon.range,
                target: None,
            });
        },
        None => {}
    }

    match &item.magic_weapon {
        Some(_magic_weapon) => {
            entity = entity.with(MagicWeapon {
                range: 12,
                target: None,
            });
        },
        None => {}
    }

    match &item.spells {
        Some(spells) => {
            entity = entity.with(SpellKnowledge {
                spells: spells.iter().map(|s| Spell {
                    name: s.name.clone(),
                    mp_cost: s.mp_cost,
                    damage: parse_dice_expression(&s.damage),
                    damage_type: s.damage_type,
                    range: s.range
                }).collect()
            });
        },
        None => {}
    }

    match &item.armor {
        Some(armor) => {
            entity = entity.with(Armor {
                defense: armor.defense,
            });
        }
        None => {}
    }

    match &item.hidden {
        Some(hidden) => {
            if *hidden {
                entity = entity.with(Hidden {});
            }
        }
        None => {}
    }

    match &item.triggerable {
        Some(triggerable) => {
            entity = entity.with(Triggerable {
                damage: triggerable.damage,
            });
        }
        None => {}
    }

    match &item.scroll {
        Some(scroll) => {
            match scroll.scroll_type {
                ScrollType::MagicMapper => {
                    entity = entity.with(MagicMapper {});
                }
            }
        },
        None => {}
    }
    return entity;
}

fn color_from_hex(hex: &str) -> Result<Color, &'static str> {
    let hex = hex.strip_prefix('#').ok_or("missing #")?;
    if hex.len() != 6 {
        return Err("invalid hex length");
    }
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "bad red")?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "bad green")?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "bad blue")?;
    Ok(Color::Rgb(r, g, b))
}

pub fn spawn_player(ecs: &mut World, x: i32, y: i32) -> Entity {
    return ecs
        .create_entity()
        .with(Position { x: x, y: y })
        .with(Renderable {
            glyph: '@',
            bg: Color::Black,
            fg: Color::Yellow,
            index: 0,
        })
        .with(Player {})
        .with(Name {
            name: "Player the unnamed".to_string(),
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
        })
        .with(BlocksTile {})
        .with(Stats {
            hp: Pool {
                current: 50,
                max: 50,
            },
            mp: Pool {
                current: 10,
                max: 10,
            },
            exp: Pool {
                current: 0,
                max: 1_000,
            },
            level: 1,
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        })
        .with(Inventory {
            gold: 10,
            items: IndexMap::new(),
            index: 0,
        })
        .with(SpellKnowledge {
            spells: vec![],
        })
        .build();
}
