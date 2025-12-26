use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::style::Color;
use rltk::RandomNumberGenerator;
use specs::prelude::*;

use crate::{
    Attack, Damage, Name, Stats,
    component::{Armor, AttackType, Equipped, Lifetime, MeleeWeapon, Position, RangedWeapon, Renderable}, logbook::logbook::Logger,
};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, RandomNumberGenerator>,
        WriteStorage<'a, Attack>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Stats>,
        WriteStorage<'a, Damage>,
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, MeleeWeapon>,
        ReadStorage<'a, RangedWeapon>,
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
            mut rng,
            mut attacks,
            names,
            stats,
            mut damages,
            equipment,
            melee_weapons,
            ranged_weapons,
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
                    let mut weapon_damage: i32 = 1;
                    match attack.attack_type {
                        AttackType::Melee => {
                            for (equipped, melee_weapon) in (&equipment, &melee_weapons).join() {
                                if equipped.owner == attacker_entity {
                                    weapon_damage = rng.roll_dice(melee_weapon.damage.dice_count, melee_weapon.damage.dice_sides)
                                        + melee_weapon.damage.modifier;
                                }
                            }
                        },
                        AttackType::Ranged => {
                            for (equipped, ranged_weapon) in (&equipment, &ranged_weapons).join() {
                                if equipped.owner == attacker_entity {
                                    weapon_damage = rng.roll_dice(ranged_weapon.damage.dice_count, ranged_weapon.damage.dice_sides)
                                        + ranged_weapon.damage.modifier;
                                }
                            }
                        }
                    }

                    let mut armor_defense = 0;
                    for (equipped, armor) in (&equipment, &armor).join() {
                        if equipped.owner == attack.target {
                            armor_defense = armor.defense;
                        }
                    }

                    let raw_damage = i32::max(0, ((stat.strength - 10) / 2) + weapon_damage);
                    let raw_defense = i32::max(0, ((target_stats.dexterity - 10) / 2) + armor_defense);
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
                    Damage::new_damage(&mut damages, Some(attacker_entity), attack.target, damage_inflicted);

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
