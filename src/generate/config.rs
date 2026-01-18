use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;

use crate::component::{DamageType, EquipmentSlot, Pool, PotionType};

#[derive(Deserialize)]
pub struct ItemConfig {
    pub name: String,
    pub description: String,
    pub base_value: i32,
    pub renderable: Option<RenderableConfig>,
    pub spawn: Option<SpawnConfig>,
    pub potion: Option<PotionConfig>,
    pub scroll: Option<ScrollConfig>,
    pub equippable: Option<EquippableConfig>,
    pub melee_weapon: Option<MeleeWeaponConfig>,
    pub ranged_weapon: Option<RangedWeaponConfig>,
    pub magic_weapon: Option<MagicWeaponConfig>,
    pub spells: Option<Vec<SpellConfig>>,
    pub armor: Option<ArmorConfig>,
    pub hidden: Option<bool>,
    pub triggerable: Option<TriggerableConfig>,
}

#[derive(Deserialize)]
pub struct MonsterConfig {
    pub name: String,
    pub description: String,
    pub renderable: Option<RenderableConfig>,
    pub spawn: Option<SpawnConfig>,
    pub viewshed: Option<ViewshedConfig>,
    pub stats: Option<StatsConfig>,
    pub drop_type: Option<DropType>,
}

#[derive(Deserialize)]
pub struct DropConfig {
    pub drop_type: DropType,
    pub drops: Vec<DropChoiceConfig>,
}

#[derive(Deserialize)]
pub struct DropChoiceConfig {
    pub name: String,
    pub weight: i32,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub enum DropType {
    Animal,
    Goblin,
    Orc
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
    pub potion_type: PotionType,
    pub restore_amount: i32,
}

#[derive(Deserialize)]
pub enum ScrollType {
    MagicMapper,
}

#[derive(Deserialize)]
pub struct ScrollConfig {
    pub scroll_type: ScrollType,
}

#[derive(Deserialize)]
pub struct EquippableConfig {
    pub slot: EquipmentSlot
}

#[derive(Deserialize)]
pub struct MeleeWeaponConfig {
    pub damage: String,
    pub damage_type: DamageType,
}

#[derive(Deserialize)]
pub struct RangedWeaponConfig {
    pub damage: String,
    pub range: i32
}

#[derive(Deserialize)]
pub struct MagicWeaponConfig {}

#[derive(Deserialize)]
pub struct SpellConfig {
    pub name: String,
    pub mp_cost: i32,
    pub damage: String,
    pub damage_type: DamageType,
    pub range: i32,
}

#[derive(Deserialize)]
pub struct ArmorConfig {
    pub defense: i32
}

#[derive(Deserialize)]
pub struct TriggerableConfig {
    pub damage: i32
}

#[derive(Deserialize)]
pub struct ViewshedConfig {
    pub range: i32
}

#[derive(Deserialize)]
pub struct StatsConfig {
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

#[derive(Debug, Clone)]
pub struct DiceExpression {
    pub dice_count: i32,
    pub dice_sides: i32,
    pub modifier: i32,
}

impl DiceExpression {
    pub fn to_expression(&self) -> String {
        return format!(
            "{}d{}{}",
            self.dice_count,
            self.dice_sides,
            if self.modifier == 0 { "".to_string() } else {
                format!(
                    "{}{}",
                    if self.modifier > 0 { "+" } else { "-" },
                    self.modifier
                )
            }
        );
    }
}

pub fn parse_dice_expression(dice : &str) -> DiceExpression {
    lazy_static! {
        static ref DICE_RE : Regex = Regex::new(r"(\d+)d(\d+)([\+\-]\d+)?").unwrap();
    }
    let mut dice_count = 1;
    let mut dice_sides = 4;
    let mut modifier = 0;
    for cap in DICE_RE.captures_iter(dice) {
        if let Some(group) = cap.get(1) {
            dice_count = group.as_str().parse::<i32>().expect("Not a digit");
        }
        if let Some(group) = cap.get(2) {
            dice_sides = group.as_str().parse::<i32>().expect("Not a digit");
        }
        if let Some(group) = cap.get(3) {
            modifier = group.as_str().parse::<i32>().expect("Not a digit");
        }

    }
    DiceExpression { dice_count, dice_sides, modifier }
}
