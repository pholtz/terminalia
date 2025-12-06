use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::style::Color;
use specs::prelude::*;

use crate::{
    Attack, Damage, Logbook, Name, Stats,
    component::{Armor, Equipped, Lifetime, MeleeWeapon, Position, Renderable},
};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Attack>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Stats>,
        WriteStorage<'a, Damage>,
        WriteExpect<'a, Logbook>,
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
            mut logbook,
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
            if stat.hp > 0 {
                // attacker's health
                let target_stats = stats.get(attack.target).unwrap();
                let target_name = names.get(attack.target).unwrap();
                if target_stats.hp > 0 {
                    // target's health

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
                    let raw_defense = target_stats.defense + armor_defense;
                    let damage_inflicted = i32::max(0, raw_damage - raw_defense);

                    if damage_inflicted == 0 {
                        logbook.entries.push(format!(
                            "{} is too weak to hurt {}",
                            &name.name, &target_name.name
                        ));
                        continue;
                    }
                    logbook.entries.push(format!(
                        "{} hits {}, inflicting {} damage",
                        &name.name, &target_name.name, damage_inflicted
                    ));
                    Damage::new_damage(&mut damages, attack.target, damage_inflicted);

                    /*
                     * Create combat particle representing an attack animation.
                     */
                    if let Some(pos) = positions.get(attack.target) {
                        entities.build_entity()
                            .with(pos.clone(), &mut positions)
                            .with(Renderable { glyph: '/', fg: Color::White, bg: Color::Gray, index: 0 }, &mut renderables)
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
