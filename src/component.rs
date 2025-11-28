use indexmap::IndexMap;
use ratatui::style::Color;
use rltk::Point;
use specs::prelude::*;
use specs_derive::Component;

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
    pub index: u8,
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

#[derive(Component, Debug)]
pub struct Name {
    pub name: String,
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

#[derive(Component)]
pub struct Inventory {
    pub gold: i32,
    pub items: IndexMap<String, Vec<Entity>>,
}

#[derive(Component, Debug)]
pub struct Item {

}

#[derive(Component, Debug)]
pub struct Potion {
    pub heal_amount: i32,
}

#[derive(Component, Debug)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component, Debug)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Component)]
pub struct Attack {
    pub target: Entity,
}

#[derive(Component)]
pub struct Damage {
    pub amount: Vec<i32>,
}

impl Damage {
    pub fn new_damage(store: &mut WriteStorage<Damage>, victim: Entity, amount: i32) {
        if let Some(damage) = store.get_mut(victim) {
            damage.amount.push(amount);
        } else {
            store
                .insert(
                    victim,
                    Damage {
                        amount: vec![amount],
                    },
                )
                .expect("Unable to insert damage");
        }
    }
}
