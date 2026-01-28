use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::style::Color;
use rltk::{Point, RandomNumberGenerator, line2d};
use specs::prelude::*;

use crate::{
    Attack, Damage, Name, Stats,
    component::{
        Armor, AttackType, DamageType, Equipped, Lifetime, MeleeWeapon, Position, RangedWeapon, Renderable
    },
    logbook::logbook::Logger,
};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, RandomNumberGenerator>,
        WriteStorage<'a, Attack>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Stats>,
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
            mut stats,
            mut damages,
            equipment,
            melee_weapons,
            ranged_weapons,
            armor,
            mut positions,
            mut renderables,
            mut lifetimes,
        ) = data;

        let mut mana_burndown: Vec<(Entity, i32)> = Vec::new();

        for (attacker_entity, attack, name, stat) in
            (&entities, &mut attacks, &names, &stats).join()
        {
            // attacker's health
            if stat.hp.current > 0 {
                if !stats.contains(attack.target) {
                    continue;
                }
                let target_stats = stats.get(attack.target).unwrap();
                let target_name = names.get(attack.target).unwrap();

                // target's health
                if target_stats.hp.current > 0 {
                    let mut weapon_damage: i32 = 1;
                    let mut weapon_name: String = "fisticuffs".to_string();
                    let mut damage_type: DamageType = DamageType::Bludgeoning;
                    match attack.attack_type {
                        AttackType::Melee => {
                            for (equipped, melee_weapon, name) in (&equipment, &melee_weapons, &names).join() {
                                if equipped.owner == attacker_entity {
                                    weapon_damage = rng.roll_dice(
                                        melee_weapon.damage.dice_count,
                                        melee_weapon.damage.dice_sides,
                                    ) + melee_weapon.damage.modifier;
                                    weapon_name = name.name.clone();
                                    damage_type = melee_weapon.damage_type;
                                }
                            }
                        }
                        AttackType::Ranged => {
                            for (equipped, ranged_weapon, name) in (&equipment, &ranged_weapons, &names).join() {
                                if equipped.owner == attacker_entity {
                                    weapon_damage = rng.roll_dice(
                                        ranged_weapon.damage.dice_count,
                                        ranged_weapon.damage.dice_sides,
                                    ) + ranged_weapon.damage.modifier;
                                    weapon_name = name.name.clone();
                                    ranged_weapon.damage_type;
                                }
                            }
                        }
                        AttackType::Magic => {
                            if let Some(spell) = &attack.spell {
                                if stat.mp.current >= spell.mp_cost {
                                    weapon_damage = rng.roll_dice(
                                        spell.damage.dice_count,
                                        spell.damage.dice_sides
                                    ) + spell.damage.modifier;
                                    weapon_name = spell.name.clone();
                                    damage_type = spell.damage_type;
                                    mana_burndown.push((attacker_entity, spell.mp_cost));
                                } else {
                                    Logger::new()
                                        .append("You tried to case a spell, but you don't have enough mana!")
                                        .log();
                                    continue;
                                }
                            } else {
                                Logger::new()
                                    .append("You tried to cast a spell, but you don't know any spells, silly!")
                                    .log();
                                continue;
                            }
                        }
                    }

                    let mut armor_defense = 0;
                    for (equipped, armor) in (&equipment, &armor).join() {
                        if equipped.owner == attack.target {
                            armor_defense = armor.defense;
                        }
                    }

                    let raw_damage = i32::max(0, stat_to_modifier(stat.strength) + weapon_damage);
                    let raw_defense =
                        i32::max(0, stat_to_modifier(stat.dexterity) + armor_defense);
                    let damage_inflicted = i32::max(0, raw_damage - raw_defense);

                    if damage_inflicted == 0 {
                        Logger::new()
                            .append(format!(
                                "{} tries to strike {} with {}, but was too weak",
                                &name.name, &target_name.name, weapon_name
                            ))
                            .log();
                        continue;
                    }
                    Logger::new()
                        .append(format!(
                            "{} hits {} with {}, inflicting {} {:?} damage",
                            &name.name, &target_name.name, weapon_name, damage_inflicted, damage_type,
                        ))
                        .log();
                    Damage::new_damage(
                        &mut damages,
                        Some(attacker_entity),
                        attack.target,
                        damage_inflicted,
                    );

                    /*
                     * Create combat particle representing an attack animation.
                     */
                    if let Some(pos) = positions.get(attack.target) {
                        match attack.attack_type {
                            AttackType::Melee => {
                                entities
                                    .build_entity()
                                    .with(pos.clone(), &mut positions)
                                    .with(
                                        Renderable {
                                            glyph: '\\',
                                            fg: Color::White,
                                            bg: Color::Gray,
                                            index: 0,
                                        },
                                        &mut renderables,
                                    )
                                    .with(
                                        Lifetime {
                                            created_at: SystemTime::now()
                                                .duration_since(UNIX_EPOCH)
                                                .expect("uhhhh")
                                                .as_millis(),
                                            lifetime_ms: 200,
                                        },
                                        &mut lifetimes,
                                    )
                                    .build();
                            }
                            AttackType::Ranged => {
                                let attacker_pos = positions
                                    .get(attacker_entity)
                                    .expect("Unable to access ranged attacker position");
                                let target_pos = positions
                                    .get(attack.target)
                                    .expect("Unable to access ranged target position");
                                let line_points = line2d(
                                    rltk::LineAlg::Bresenham,
                                    Point {
                                        x: attacker_pos.x,
                                        y: attacker_pos.y,
                                    },
                                    Point {
                                        x: target_pos.x,
                                        y: target_pos.y,
                                    },
                                );
                                let mut prev_point: Option<Point> = None;
                                for point in line_points.iter() {
                                    let glyph = prev_point
                                        .map(|prev| {
                                            generate_directional_ranged_attack_glyph(prev, *point)
                                        })
                                        .unwrap_or('-');
                                    entities
                                        .build_entity()
                                        .with(
                                            Position {
                                                x: point.x,
                                                y: point.y,
                                            },
                                            &mut positions,
                                        )
                                        .with(
                                            Renderable {
                                                glyph: glyph,
                                                fg: Color::White,
                                                bg: Color::default(),
                                                index: 0,
                                            },
                                            &mut renderables,
                                        )
                                        .with(
                                            Lifetime {
                                                created_at: SystemTime::now()
                                                    .duration_since(UNIX_EPOCH)
                                                    .expect("uhhhh")
                                                    .as_millis(),
                                                lifetime_ms: 100,
                                            },
                                            &mut lifetimes,
                                        )
                                        .build();
                                    prev_point = Some(*point);
                                }
                            }
                            AttackType::Magic => {
                                let attacker_pos = positions
                                    .get(attacker_entity)
                                    .expect("Unable to access magic attacker position");
                                let target_pos = positions
                                    .get(attack.target)
                                    .expect("Unable to access magic target position");
                                let line_points = line2d(
                                    rltk::LineAlg::Bresenham,
                                    Point {
                                        x: attacker_pos.x,
                                        y: attacker_pos.y,
                                    },
                                    Point {
                                        x: target_pos.x,
                                        y: target_pos.y,
                                    },
                                );
                                for point in line_points.iter() {
                                    entities
                                        .build_entity()
                                        .with(
                                            Position {
                                                x: point.x,
                                                y: point.y,
                                            },
                                            &mut positions,
                                        )
                                        .with(
                                            Renderable {
                                                glyph: '*',
                                                fg: Color::White,
                                                bg: Color::default(),
                                                index: 0,
                                            },
                                            &mut renderables,
                                        )
                                        .with(
                                            Lifetime {
                                                created_at: SystemTime::now()
                                                    .duration_since(UNIX_EPOCH)
                                                    .expect("uhhhh")
                                                    .as_millis(),
                                                lifetime_ms: 100,
                                            },
                                            &mut lifetimes,
                                        )
                                        .build();
                                }
                            }
                        }
                    }
                }
            }
        }
        attacks.clear();

        /*
         * Commit the mana burndown to ecs.
         */
        for (attacker_entity, mana_cost) in mana_burndown.iter() {
            let stat = stats.get_mut(*attacker_entity).expect("Unable to access magic attacker stats");
            stat.mp.current -= mana_cost;
        }
    }
}

pub fn stat_to_modifier(stat: i32) -> i32 {
    return (stat - 10) / 2;
}

fn generate_directional_ranged_attack_glyph(previous: Point, current: Point) -> char {
    if previous.x == current.x {
        return '|';
    }

    if previous.y == current.y {
        return '-';
    }

    if previous.x < current.x && previous.y < current.y {
        return '\\';
    }

    if previous.x < current.x && previous.y > current.y {
        return '/';
    }

    if previous.x > current.x && previous.y < current.y {
        return '/';
    }

    if previous.x > current.x && previous.y > current.y {
        return '\\';
    }
    return '-';
}
