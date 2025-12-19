use std::{fs, sync::Mutex};

use indexmap::IndexMap;
use lazy_static::lazy_static;
use ratatui::style::Color;
use rltk::{RandomNumberGenerator};
use serde::Deserialize;
use specs::{prelude::*};

use crate::{component::{
    Armor, BlocksTile, EquipmentSlot, Equippable, Hidden, Inventory, Item, MagicMapper, MeleeWeapon, Monster, Name, Player, Pool, Position, Potion, Renderable, Stats, Triggerable, Viewshed
}, generate::{random_table::RandomTable, rect::Rect}};

#[derive(Deserialize)]
pub struct ItemConfig {
    pub name: String,
    pub description: String,
    pub renderable: Option<RenderableConfig>,
    pub spawn: Option<SpawnConfig>,
    pub potion: Option<PotionConfig>,
    pub equippable: Option<EquippableConfig>,
    pub melee_weapon: Option<MeleeWeaponConfig>,
}

#[derive(Deserialize)]
pub struct RenderableConfig {
    pub glyph: String,
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub index: u8,
}

#[derive(Deserialize)]
pub struct SpawnConfig {
    pub min_floor: i32,
    pub base_weight: i32,
}

#[derive(Deserialize)]
pub struct PotionConfig {
    pub heal_amount: i32,
}

#[derive(Deserialize)]
pub struct EquippableConfig {
    pub slot: EquipmentSlot
}

#[derive(Deserialize)]
pub struct MeleeWeaponConfig {
    pub damage: i32
}

lazy_static! {
    pub static ref ITEMS: Mutex<Vec<ItemConfig>> = Mutex::new(Vec::new());
    pub static ref MONSTERS: Mutex<Vec<Item>> = Mutex::new(Vec::new());
}

pub fn initialize_config() {
    let items_raw = fs::read_to_string("./config/items.json").unwrap();
    let items: Vec<ItemConfig> = serde_json::from_str(&items_raw).unwrap();
    ITEMS.lock().unwrap().extend(items);
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
                Some(spawn) => { item_spawn_table.push(item.name.clone(), spawn.base_weight); },
                None => {},
            };
        }
        // let item_spawn_table = RandomTable::new()
        //     .add("Health Potion", 25)
        //     .add("Magic Mapping Scroll", 5)
        //     .add("Dagger", 4 + floor_index as i32)
        //     .add("Battered Shield", 6 + floor_index as i32)
        //     .add("Basic Trap", 10 + floor_index as i32);

        (pos, item_spawn_table.roll(&mut rng))
    };

    for item in ITEMS.lock().unwrap().iter() {
        if item.name != spawn { continue }
        let mut entity = ecs.create_entity()
            .with(pos)
            .with(Name { name: item.name.clone() })
            .with(Item { description: item.description.clone() });
        match &item.renderable {
            Some(renderable) => {
                entity = entity.with(Renderable {
                    glyph: renderable.glyph.chars().next().unwrap_or('!'),
                    fg: renderable.fg.clone()
                        .map(|fg| color_from_hex(fg.as_str()).unwrap())
                        .unwrap_or(Color::default()),
                    bg: renderable.bg.clone()
                        .map(|bg| color_from_hex(bg.as_str()).unwrap())
                        .unwrap_or(Color::default()),
                    index: renderable.index,
                });
            },
            None => {},
        }
        match &item.potion {
            Some(potion) => {
                entity = entity.with(Potion {
                    heal_amount: potion.heal_amount
                });
            },
            None => {},
        }
        match &item.equippable {
            Some(equippable) => {
                entity = entity.with(Equippable {
                    slot: equippable.slot
                });
            },
            None => {},
        }
        match &item.melee_weapon {
            Some(melee_weapon) => {
                entity = entity.with(MeleeWeapon {
                    damage: melee_weapon.damage
                });
            },
            None => {},
        }
        entity.build();
    }

    // match spawn.as_ref() {
    //     "Health Potion" => spawn_potion_health(ecs, pos),
    //     "Magic Mapping Scroll" => spawn_scroll_magic_mapping(ecs, pos),
    //     "Dagger" => spawn_dagger(ecs, pos),
    //     "Battered Shield" => spawn_shield(ecs, pos),
    //     "Basic Trap" => spawn_trap_basic(ecs, pos),
    //     _ => {},
    // }
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


/// Spawns a weighted monster based on the current floor and internal spawn table.
pub fn spawn_weighted_monster(ecs: &mut World, floor_index: u32, room: &Rect) {
    let (pos, spawn): (Position, String) = {
        let mut rng = ecs.fetch_mut::<RandomNumberGenerator>();
        let width = room.x2 - room.x1;
        let height = room.y2 - room.y1;
        let x = room.x1 + rng.roll_dice(1, width - 1);
        let y = room.y1 + rng.roll_dice(1, height - 1);
        let pos = Position { x: x, y: y };
        let monster_spawn_table = RandomTable::new()
            .add("Rat", 10)
            .add("Snake", 8)
            .add("Bat", 8)
            .add("Goblin", 1 + floor_index as i32);

        (pos, monster_spawn_table.roll(&mut rng))
    };

    match spawn.as_ref() {
        "Rat" => spawn_monster_rat(ecs, pos),
        "Snake" => spawn_monster_snake(ecs, pos),
        "Bat" => spawn_monster_bat(ecs, pos),
        "Goblin" => spawn_monster_goblin(ecs, pos),
        _ => {},
    }
}

pub fn spawn_player(ecs: &mut World, x: i32, y: i32) -> Entity {
    return ecs.create_entity()
        .with(Position { x: x, y: y })
        .with(Renderable {
            glyph: '@',
            bg: Color::Black,
            fg: Color::Yellow,
            index: 0,
        })
        .with(Player {})
        .with(Name {
            name: "player".to_string(),
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
        })
        .with(BlocksTile {})
        .with(Stats {
            hp: Pool { current: 50, max: 50 },
            mp: Pool { current: 10, max: 10 },
            exp: Pool { current: 0, max: 1_000 },
            level: 1,
            strength: 5,
            dexterity: 1,
            constitution: 1,
            intelligence: 1,
            wisdom: 1,
            charisma: 1,
        })
        .with(Inventory {
            gold: 0,
            items: IndexMap::new(),
            index: 0,
        })
        .build();
}

pub fn spawn_potion_health(ecs: &mut World, pos: Position) {
    ecs.create_entity()
        .with(pos)
        .with(Renderable {
            glyph: 'i',
            fg: Color::Cyan,
            bg: Color::Black,
            index: 2,
        })
        .with(Name {
            name: "Potion of pathetically minor healing".to_string(),
        })
        .with(Item {
            description: "A glowing red vial of an unknown substance. Smells delicious.".to_string(),
        })
        .with(Potion { heal_amount: 10 })
        .build();
}

pub fn spawn_scroll_magic_mapping(ecs: &mut World, pos: Position) {
    ecs.create_entity()
        .with(pos)
        .with(Renderable {
            glyph: ']',
            fg: Color::LightMagenta,
            bg: Color::Black,
            index: 2,
        })
        .with(Name {
            name: "Scroll of magic mapping".to_string(),
        })
        .with(Item {
            description: "An ancient looking, mysterious scroll that glows with a faint white light. Undecipherable.".to_string(),
        })
        .with(MagicMapper {})
        .build();
}

pub fn spawn_dagger(ecs: &mut World, pos: Position) {
    ecs.create_entity()
        .with(pos)
        .with(Renderable {
            glyph: '/',
            fg: Color::Gray,
            bg: Color::Black,
            index: 2,
        })
        .with(Name { name: "Dagger".to_string() })
        .with(Item { description: "A short, pointy blade made for quick cuts.".to_string() })
        .with(Equippable { slot: EquipmentSlot::Weapon })
        .with(MeleeWeapon { damage: 2 })
        .build();
}

pub fn spawn_shield(ecs: &mut World, pos: Position) {
    ecs.create_entity()
        .with(pos)
        .with(Renderable {
            glyph: '0',
            fg: Color::Gray,
            bg: Color::Black,
            index: 2
        })
        .with(Name { name: "Battered Shield".to_string() })
        .with(Item { description:
            "A medium-sized, circular shield with some sizeable dents.
            Seems well made, though."
            .to_string()
        })
        .with(Equippable { slot: EquipmentSlot::Shield })
        .with(Armor { defense: 1 })
        .build();
}

pub fn spawn_trap_basic(ecs: &mut World, pos: Position) {
    ecs.create_entity()
        .with(pos)
        .with(Renderable {
            glyph: '^',
            fg: Color::Red,
            bg: Color::Black,
            index: 2,
        })
        .with(Name { name: "Basic Trap".to_string() })
        .with(Hidden {})
        .with(Triggerable { damage: 8 })
        .build();
}

pub fn spawn_monster_rat(ecs: &mut World, pos: Position) {
    ecs.create_entity()
        .with(pos)
        .with(Renderable {
            glyph: 'r',
            bg: Color::Black,
            fg: Color::Red,
            index: 1,
        })
        .with(Monster {})
        .with(Name {
            name: "rat".to_string(),
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 6,
        })
        .with(BlocksTile {})
        .with(Stats {
            hp: Pool { current: 4, max: 4 },
            mp: Pool { current: 0, max: 0 },
            exp: Pool { current: 0, max: 0 },
            level: 1,
            strength: 2,
            dexterity: 0,
            constitution: 1,
            intelligence: 1,
            wisdom: 1,
            charisma: 1,
        })
        .build();
}

pub fn spawn_monster_snake(ecs: &mut World, pos: Position) {
    ecs.create_entity()
        .with(pos)
        .with(Renderable {
            glyph: 's',
            bg: Color::Black,
            fg: Color::Red,
            index: 1,
        })
        .with(Monster {})
        .with(Name {
            name: "snake".to_string(),
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
        })
        .with(BlocksTile {})
        .with(Stats {
            hp: Pool { current: 8, max: 8 },
            mp: Pool { current: 0, max: 0 },
            exp: Pool { current: 0, max: 0 },
            level: 1,
            strength: 2,
            dexterity: 1,
            constitution: 1,
            intelligence: 1,
            wisdom: 1,
            charisma: 1,
        })
        .build();
}

pub fn spawn_monster_bat(ecs: &mut World, pos: Position) {
    ecs.create_entity()
        .with(pos)
        .with(Renderable {
            glyph: 'w',
            bg: Color::Black,
            fg: Color::Red,
            index: 1,
        })
        .with(Monster {})
        .with(Name { name: "bat".to_string() })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 10
        })
        .with(BlocksTile {})
        .with(Stats {
            hp: Pool { current: 10, max: 10 },
            mp: Pool { current: 0, max: 0 },
            exp: Pool { current: 0, max: 0 },
            level: 1,
            strength: 2,
            dexterity: 1,
            constitution: 1,
            intelligence: 1,
            wisdom: 1,
            charisma: 1,
        })
        .build();
}

pub fn spawn_monster_goblin(ecs: &mut World, pos: Position) {
    ecs.create_entity()
        .with(pos)
        .with(Renderable {
            glyph: 'g',
            bg: Color::Black,
            fg: Color::Red,
            index: 1,
        })
        .with(Monster {})
        .with(Name {
            name: "goblin".to_string(),
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 12,
        })
        .with(BlocksTile {})
        .with(Stats {
            hp: Pool { current: 12, max: 12 },
            mp: Pool { current: 0, max: 0 },
            exp: Pool { current: 0, max: 0 },
            level: 2,
            strength: 3,
            dexterity: 1,
            constitution: 1,
            intelligence: 1,
            wisdom: 1,
            charisma: 1,
        })
        .build();
}
