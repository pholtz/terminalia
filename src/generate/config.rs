use serde::Deserialize;

use crate::component::{EquipmentSlot, Pool};

#[derive(Deserialize)]
pub struct ItemConfig {
    pub name: String,
    pub description: String,
    pub renderable: Option<RenderableConfig>,
    pub spawn: Option<SpawnConfig>,
    pub potion: Option<PotionConfig>,
    pub equippable: Option<EquippableConfig>,
    pub melee_weapon: Option<MeleeWeaponConfig>,
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
