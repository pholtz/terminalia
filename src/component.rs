use indexmap::IndexMap;
use ratatui::style::Color;
use rltk::Point;
use serde::Deserialize;
use specs::prelude::*;
use specs_derive::Component;

use crate::generate::config::{DiceExpression, DropType};

/**
 * All supporting enums defined below.
 */
#[derive(PartialEq, Copy, Clone, Debug, Deserialize)]
pub enum EquipmentSlot {
    Weapon,
    Shield,
    Head,
    Chest,
    Hands,
    Legs,
    Feet,
}

#[derive(PartialEq, Copy, Clone, Debug, Deserialize)]
pub enum DamageType {
    Piercing,
    Slashing,
    Bludgeoning,
    Fire,
}

#[derive(PartialEq)]
pub enum AttackType {
    Melee,
    Ranged,
    Magic,
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub enum PotionType {
    Health,
    Mana
}

/**
 * All specs components defined below.
 */
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
pub struct Monster {
    pub drop_type: Option<DropType>
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

#[derive(Debug, Clone, Deserialize)]
pub struct Pool {
    pub current: i32,
    pub max: i32,
}

#[derive(Component)]
pub struct Stats {
    pub hp: Pool,
    pub mp: Pool,
    pub exp: Pool,
    pub level: i32,

    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

#[derive(Component)]
pub struct Inventory {
    pub gold: i32,
    pub items: IndexMap<String, Vec<Entity>>,
    pub index: usize,
}

#[derive(Component, Debug)]
pub struct Item {
    pub description: String,
    pub base_value: i32,
}

#[derive(Component, Debug)]
pub struct Potion {
    pub potion_type: PotionType,
    pub restore_amount: i32,
}

#[derive(Component, Debug)]
pub struct Equippable {
    pub slot: EquipmentSlot
}

#[derive(Component, Debug)]
pub struct Equipped {
    pub slot: EquipmentSlot,
    pub owner: Entity,
}

#[derive(Component, Debug)]
pub struct MeleeWeapon {
    pub damage: DiceExpression,
}

#[derive(Component, Debug)]
pub struct RangedWeapon {
    pub damage: DiceExpression,
    pub range: i32,
    pub target: Option<Entity>,
}

#[derive(Component, Debug)]
pub struct MagicWeapon {
    pub range: i32,
    pub target: Option<Entity>,
}

#[derive(Component, Debug)]
pub struct Armor {
    pub defense: i32,
}

#[derive(Component, Debug, Clone)]
pub struct Spell {
    pub name: String,
    pub mp_cost: i32,
    pub damage: DiceExpression,
    pub damage_type: DamageType,
    pub range: i32,
}

#[derive(Component, Debug)]
pub struct SpellKnowledge {
    pub spells: Vec<Spell>,
}

#[derive(Component, Debug)]
pub struct MagicMapper {}

#[derive(Component, Debug)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component, Debug)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub items: Vec<Entity>,
}

#[derive(Component, Debug)]
pub struct WantsToConsumeItem {
    pub item: Entity,
}

#[derive(Component)]
pub struct Attack {
    pub attack_type: AttackType,
    pub target: Entity,
    pub spell: Option<Spell>,
}

#[derive(Component)]
pub struct Damage {
    pub amount: Vec<i32>,
    pub attacker: Option<Entity>,
}

impl Damage {
    pub fn new_damage(store: &mut WriteStorage<Damage>, attacker: Option<Entity>, victim: Entity, amount: i32) {
        if let Some(damage) = store.get_mut(victim) {
            damage.amount.push(amount);
        } else {
            store
                .insert(
                    victim,
                    Damage {
                        amount: vec![amount],
                        attacker: attacker,
                    },
                )
                .expect("Unable to insert damage");
        }
    }
}

#[derive(Component, Debug)]
pub struct Experience {
    pub amount: Vec<i32>
}

impl Experience {
    pub fn new(store: &mut WriteStorage<Experience>, recipient: Entity, amount: i32) {
        if let Some(experience) = store.get_mut(recipient) {
            experience.amount.push(amount);
        } else {
            store.insert(recipient, Experience { amount: vec![amount] }).expect("Unable to award experience");
        }
    }
}

#[derive(Component, Debug)]
pub struct Lifetime {
    pub created_at: u128,
    pub lifetime_ms: u128,
}

#[derive(Component, Debug)]
pub struct Hidden {

}

#[derive(Component, Debug)]
pub struct Triggerable {
    pub damage: i32,
}

#[derive(Component, Debug)]
pub struct Npc {
    pub dialogue: Option<Vec<String>>,
}

#[derive(Component, Debug)]
pub struct Vendor {
    pub items: Vec<Entity>,
}
