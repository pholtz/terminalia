use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::style::Color;
use specs::prelude::*;

use crate::{
    Attack, Damage, Name, Stats,
    component::{Armor, Equipped, Lifetime, MeleeWeapon, Position, Renderable}, logbook::logbook::Logger,
};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Attack>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Stats>,
        WriteStorage<'a, Damage>,
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, MeleeWeapon>,
        ReadStorage<'a, Armor>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, Lifetime>,
    );

    /*
     * Query each attack by the attacker.
     *
     * The target is contained within the Attack entity itself.
     * Before applying any damage, we should make sure that both
     * the attacker and the victim are still alive.
     */
    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut attacks,
            names,
            stats,
            mut damages,
            equipment,
            melee_weapons,
            armor,
            mut positions,
            mut renderables,
            mut lifetimes,
        ) = data;

        for (attacker_entity, attack, name, stat) in
            (&entities, &mut attacks, &names, &stats).join()
        {
            // attacker's health
            if stat.hp.current > 0 {
                let target_stats = stats.get(attack.target).unwrap();
                let target_name = names.get(attack.target).unwrap();
                
                // target's health
                if target_stats.hp.current > 0 {
                    let mut melee_weapon_damage = 0;
                    for (equipped, melee_weapon) in (&equipment, &melee_weapons).join() {
                        if equipped.owner == attacker_entity {
                            melee_weapon_damage = melee_weapon.damage;
                        }
                    }

                    let mut armor_defense = 0;
                    for (equipped, armor) in (&equipment, &armor).join() {
                        if equipped.owner == attack.target {
                            armor_defense = armor.defense;
                        }
                    }

                    let raw_damage = stat.strength + melee_weapon_damage;
                    let raw_defense = target_stats.dexterity + armor_defense;
                    let damage_inflicted = i32::max(0, raw_damage - raw_defense);

                    if damage_inflicted == 0 {
                        Logger::new()
                            .append(format!("{} is too weak to hurt {}", &name.name, &target_name.name))
                            .log();
                        continue;
                    }
                    Logger::new()
                        .append(format!(
                            "{} hits {}, inflicting {} damage",
                            &name.name, &target_name.name, damage_inflicted
                        ))
                        .log();
                    Damage::new_damage(&mut damages, attack.target, damage_inflicted);

                    /*
                     * Create combat particle representing an attack animation.
                     */
                    if let Some(pos) = positions.get(attack.target) {
                        entities.build_entity()
                            .with(pos.clone(), &mut positions)
                            .with(Renderable { glyph: '\\', fg: Color::White, bg: Color::Gray, index: 0 }, &mut renderables)
                            .with(Lifetime {
                                created_at: SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .expect("uhhhh")
                                    .as_millis(),
                                lifetime_ms: 200,
                            }, &mut lifetimes)
                            .build();
                    }
                }
            }
        }
        attacks.clear();
    }
}
