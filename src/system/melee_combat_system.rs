use specs::prelude::*;

use crate::{Attack, Damage, Logbook, Name, Stats};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Attack>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Stats>,
        WriteStorage<'a, Damage>,
        WriteExpect<'a, Logbook>,
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
        ) = data;

        for (_entity, attack, name, stat) in (&entities, &mut attacks, &names, &stats).join() {
            if stat.hp > 0 {
                let target_stats = stats.get(attack.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(attack.target).unwrap();
                    let damage_inflicted = i32::max(0, stat.strength - target_stats.defense);
                    
                    if damage_inflicted == 0 {
                        logbook.entries.push(format!("{} is too weak to hurt {}", &name.name, &target_name.name));
                        continue;
                    }
                    logbook.entries.push(format!("{} hits {}, inflicting {} damage", &name.name, &target_name.name, damage_inflicted));
                    Damage::new_damage(&mut damages, attack.target, damage_inflicted);
                }
            }
        }
        attacks.clear();
    }
}
