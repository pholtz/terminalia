use indexmap::IndexMap;
use ratatui::style::Color;
use specs::prelude::*;

use crate::component::{
    BlocksTile, Inventory, Item, Monster, Name, Player, Position, Potion, Renderable, Stats,
    Viewshed,
};

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
            max_hp: 50,
            hp: 50,
            strength: 5,
            defense: 1,
        })
        .with(Inventory { gold: 0, items: IndexMap::new() })
        .build();
}

pub fn spawn_potion_health(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x: x, y: y })
        .with(Renderable {
            glyph: 'i',
            fg: Color::Cyan,
            bg: Color::Black,
            index: 2,
        })
        .with(Name {
            name: "Potion of health".to_string(),
        })
        .with(Item {})
        .with(Potion { heal_amount: 10 })
        .build();
}

pub fn spawn_monster_rat(ecs: &mut World, pos: Position) {
    ecs.create_entity()
        .with(Position { x: pos.x, y: pos.y })
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
            range: 8,
        })
        .with(BlocksTile {})
        .with(Stats {
            max_hp: 4,
            hp: 4,
            strength: 2,
            defense: 0,
        })
        .build();
}

pub fn spawn_monster_snake(ecs: &mut World, pos: Position) {
    ecs.create_entity()
        .with(Position { x: pos.x, y: pos.y })
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
            range: 12,
        })
        .with(BlocksTile {})
        .with(Stats {
            max_hp: 8,
            hp: 8,
            strength: 2,
            defense: 1,
        })
        .build();
}
