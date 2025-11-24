use specs::{Entity, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::component::{InBackpack, Logbook, Name, Position, WantsToPickupItem};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
        WriteExpect<'a, Logbook>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut wants_pickup,
            mut positions,
            names,
            mut backpack,
            mut logbook,
        ) = data;

        for (pickup) in (&wants_pickup).join() {
            positions.remove(pickup.item);
            backpack.insert(
                pickup.item,
                InBackpack { owner: pickup.collected_by }
            ).expect("Unable to add item to backpack");
            
            if pickup.collected_by == *player_entity {
                logbook.entries.push(format!("You pick up the {}.", names.get(pickup.item).unwrap().name));
            }
        }
        wants_pickup.clear();
    }
}