use specs::{Entity, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::component::{InBackpack, Inventory, Logbook, Name, Position, WantsToPickupItem};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
        WriteExpect<'a, Logbook>,
        WriteStorage<'a, Inventory>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut wants_pickup,
            mut positions,
            names,
            mut backpack,
            mut logbook,
            mut inventory,
        ) = data;

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
    }
}