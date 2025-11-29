use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::component::{InBackpack, Inventory, Logbook, Name, Position, Potion, Stats, WantsToConsumeItem, WantsToPickupItem};

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
        WriteExpect<'a, Logbook>,
        WriteStorage<'a, Inventory>,
        ReadStorage<'a, Potion>,
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
            mut logbook,
            mut inventory,
            potions,
        ) = data;

        /*
         * Item collection subsystem
         */
        for (pickup, _name) in (&wants_pickup, &names).join() {
            positions.remove(pickup.item);
            backpack.insert(
                pickup.item,
                InBackpack { owner: pickup.collected_by }
            ).expect("Unable to add item to backpack");

            let item_name = names.get(pickup.item).expect("Unable to access name for picked up item");

            if let Some(inv) = inventory.get_mut(pickup.collected_by) {
                let item = inv.items.entry(item_name.name.clone()).or_insert(vec![]);
                item.push(pickup.item);
            }
            
            if pickup.collected_by == *player_entity {
                logbook.entries.push(format!("You pick up the {}.", item_name.name));
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
            let item_name = names.get(consume.item).expect("Unable to access name for consumed item");

            // Someone wants to drink a potion...
            if let Some(potion) = potions.get(consume.item) {
                stat.hp = i32::min(stat.max_hp, stat.hp + potion.heal_amount);
                if entity == *player_entity {
                    logbook.entries.push(format!("You consume the {}, healing {} hp.", item_name.name, potion.heal_amount));
                }
            }
        }
        wants_consume.clear();
    }
}