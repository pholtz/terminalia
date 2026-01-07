use ratatui::style::Color;
use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{
    component::{
        Equippable, Equipped, InBackpack, Inventory, MagicMapper, Name, Position, Potion, Spell, SpellKnowledge, Stats, WantsToConsumeItem, WantsToPickupItem
    },
    generate::map::Map, logbook::logbook::Logger,
};

pub struct InventorySystem {}

impl<'a> System<'a> for InventorySystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, WantsToConsumeItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Stats>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, Inventory>,
        WriteExpect<'a, Map>,
        ReadStorage<'a, Potion>,
        WriteStorage<'a, SpellKnowledge>,
        ReadStorage<'a, MagicMapper>,
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, Equipped>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            player_entity,
            mut wants_pickup,
            mut wants_consume,
            mut positions,
            names,
            mut stats,
            mut backpack,
            mut inventories,
            mut map,
            potions,
            mut spell_knowledge,
            magic_mappers,
            equippables,
            mut equipment,
        ) = data;

        /*
         * Item collection subsystem
         */
        for (pickup, _name) in (&wants_pickup, &names).join() {
            positions.remove(pickup.item);
            backpack
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                )
                .expect("Unable to add item to backpack");

            let item_name = names
                .get(pickup.item)
                .expect("Unable to access name for picked up item");

            if let Some(inventory) = inventories.get_mut(pickup.collected_by) {
                let item_stack = inventory
                    .items
                    .entry(item_name.name.clone())
                    .or_insert(vec![]);
                item_stack.push(pickup.item);
            }

            if pickup.collected_by == *player_entity {
                Logger::new()
                    .append("You pick up the ")
                    .with_color(Color::LightBlue)
                    .append(format!("{}.", item_name.name))
                    .log();
            }
        }
        wants_pickup.clear();

        /*
         * Item consumption subsystem
         *
         * Iterates over the list of consumable components and then clears them.
         * Each consumable entity may or may not have an effect, if so it should
         * be explicitly mentioned and handled here, e.g. potion drinking.
         */
        for (entity, consume, stat) in (&entities, &wants_consume, &mut stats).join() {
            let item_name = names
                .get(consume.item)
                .expect("Unable to access name for consumed item");
            let mut has_effect = false;
            let mut should_consume = false;

            // Someone wants to drink a potion...
            if let Some(potion) = potions.get(consume.item) {
                has_effect = true;
                should_consume = true;
                stat.hp.current = i32::min(stat.hp.max, stat.hp.current + potion.heal_amount);
                if entity == *player_entity {
                    Logger::new()
                        .append("You consume the ")
                        .append_with_color(Color::Blue, format!("{}", item_name.name))
                        .append(", healing ")
                        .append_with_color(Color::Green, format!("{} hp.", potion.heal_amount))
                        .log();
                }
            }

            // Someone wants to read a spellbook...
            let mut new_spells: Vec<Spell> = Vec::new();
            if let Some(spellbook) = spell_knowledge.get(consume.item) {
                has_effect = true;
                should_consume = true;
                if entity == *player_entity {
                    new_spells.extend_from_slice(&spellbook.spells);
                    let spell_names: Vec<String> = spellbook.spells.iter().map(|s| s.name.clone()).collect();
                    Logger::new()
                        .append("You read the ")
                        .append_with_color(Color::Blue, format!("{}", item_name.name))
                        .append(", and learn the following spell(s) -> ")
                        .append_with_color(Color::Green, format!("{}", spell_names.join(", ")))
                        .log();
                }
            }

            // If the player read a spellbook above, teach any new spells to the player
            if let Some(player_spells) = spell_knowledge.get_mut(*player_entity) {
                let net_new_spells: Vec<Spell> = new_spells.iter()
                    .filter(|ns| player_spells.spells.iter().find(|os| ns.name == os.name).is_none())
                    .cloned()
                    .collect();
                player_spells.spells.extend_from_slice(&net_new_spells);
            }

            // Someone wants to equip an item...
            if let Some(equippable) = equippables.get(consume.item) {
                has_effect = true;

                let mut unequip: Vec<Entity> = Vec::new();
                for (item_entity, equipment, name) in (&entities, &equipment, &names).join() {
                    if equipment.owner == entity && equipment.slot == equippable.slot {
                        unequip.push(item_entity);
                        Logger::new().append(format!(
                            "You unequp the {} from the {:?} slot.",
                            name.name, equipment.slot,
                        )).log();
                    }
                }
                unequip.iter().for_each(|item| { equipment.remove(*item).expect("Unable to unequip item"); });

                /*
                 * If the same item is being used and is already equipped...
                 * That means the player wants to unequip it, so we should not reequip it
                 */
                let unequip_only = unequip.contains(&consume.item);

                if !unequip_only {
                    equipment.insert(consume.item, Equipped { slot: equippable.slot, owner: entity })
                        .expect("Unable to equip desired item");
                    if entity == *player_entity {
                        Logger::new().append(format!(
                            "You equip the {} to the {:?} slot.",
                            item_name.name, equippable.slot
                        )).log();
                    }
                }
            }

            // Someone wants to use a magic mapper scroll...
            if magic_mappers.contains(consume.item) {
                has_effect = true;
                should_consume = true;
                for tile in map.revealed_tiles.iter_mut() {
                    *tile = true;
                }
                Logger::new().append("The darkness lifts, and you become more aware of everything around you.").log();
            }

            if !has_effect {
                Logger::new().append(format!(
                    "You consume the {}, but nothing happens.",
                    item_name.name
                )).log();
            }

            /*
             * Decrement the item stack since it was used.
             * If this was the final element in the stack, remove the item entirely.
             */
            if should_consume {
                if let Some(inventory) = inventories.get_mut(entity) {
                    let item_stack = inventory
                        .items
                        .entry(item_name.name.clone())
                        .or_insert(vec![]);
                    item_stack.pop();
                    if item_stack.is_empty() {
                        inventory.items.shift_remove(&item_name.name);
                        if inventory.index > 0 {
                            inventory.index -= 1;
                        }
                    }
                }
            }
        }
        wants_consume.clear();
    }
}
