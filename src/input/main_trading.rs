use log::info;
use ratatui::style::Color;
use specs::prelude::*;
use crossterm::event::{KeyCode, KeyEvent};

use crate::{App, RunState, Screen, component::{Inventory, Item, Name, Vendor, WantsToPickupItem}, logbook::logbook::Logger};

pub fn handle_main_trading_key_event(
    app: &mut App,
    key_event: KeyEvent,
    vendor_entity: Entity,
    vendor_index: usize,
    player_index: usize,
    is_buying: bool,
) -> Option<RunState> {
    match key_event.code {
        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
            match is_buying {
                true => {
                    if vendor_index > 0 {
                        app.screen = Screen::Trading {
                            vendor: vendor_entity,
                            vendor_index: vendor_index - 1,
                            player_index: player_index,
                            is_buying: is_buying,
                        };
                    }
                }
                false => {
                    if player_index > 0 {
                        app.screen = Screen::Trading {
                            vendor: vendor_entity,
                            vendor_index: vendor_index,
                            player_index: player_index - 1,
                            is_buying: is_buying,
                        };
                    }
                }
            }
            None
        }

        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
            let player_entity = app.ecs.fetch::<Entity>();
            let vendors = app.ecs.read_storage::<Vendor>();
            let inventories = app.ecs.read_storage::<Inventory>();
            let vendor = vendors.get(vendor_entity).expect("Unable to access given vendor");
            let inventory = inventories.get(*player_entity).expect("Unable to retrieve the player's inventory!");
            match is_buying {
                true => {
                    if vendor_index + 1 < vendor.items.len() {
                        app.screen = Screen::Trading {
                            vendor: vendor_entity,
                            vendor_index: vendor_index + 1,
                            player_index: player_index,
                            is_buying: is_buying,
                        };
                    }
                }
                false => {
                    if player_index + 1 < inventory.items.len() {
                        app.screen = Screen::Trading {
                            vendor: vendor_entity,
                            vendor_index: vendor_index,
                            player_index: player_index + 1,
                            is_buying: is_buying,
                        };
                    }
                }
            }
            None
        }

        /*
         * Switch between buying and selling (switches highlighted list)
         */
        KeyCode::Tab => {
            app.screen = Screen::Trading {
                vendor: vendor_entity,
                vendor_index: vendor_index,
                player_index: player_index,
                is_buying: !is_buying
            };
            None
        }

        /*
         * Attempt to buy the currently selected item
         */
        KeyCode::Enter | KeyCode::Char(' ') => {
            try_buy_item(app, vendor_entity, vendor_index, player_index, is_buying)
        }

        /*
         * Exit the trading menu
         */
        KeyCode::Esc => {
            app.screen = Screen::Explore;
            None
        }

        _ => None
    }
}

/// Given a vendor and a current trading index, examine the item value
/// and the player's current gold and determine if the item can be purchased.
/// 
/// If so, add  a `WantsToPickupItem` component to the player and decrement
/// their gold. Lastly, remove the item from the vendor's inventory and reset
/// the trading index.
/// 
/// If not, add a logbook message regarding insufficient funds.
fn try_buy_item(
    app: &mut App,
    vendor_entity: Entity,
    vendor_index: usize,
    player_index: usize,
    is_buying: bool,
) -> Option<RunState> {
    let names = app.ecs.read_storage::<Name>();
    let mut items = app.ecs.write_storage::<Item>();
    let mut pickups = app.ecs.write_storage::<WantsToPickupItem>();
    let player_entity = app.ecs.fetch::<Entity>();

    // This is done in a scope to prevent borrow errors later when we mutate these components
    let item_entity = {
        let vendors = app.ecs.read_storage::<Vendor>();
        let inventories = app.ecs.read_storage::<Inventory>();
        match is_buying {
            true => vendors.get(vendor_entity)
                .expect("Unable to access vendor component during trading")
                .items.get(vendor_index)
                .copied(),
            false => inventories.get(*player_entity)
                .expect("Unable to access player inventory during trading")
                .items.get_index(player_index)
                .map(|entry| entry.1.first().expect("ahhhh"))
                .copied()
        }
    };
    if item_entity.is_none() {
        return None;
    }

    let mut vendors = app.ecs.write_storage::<Vendor>();
    let mut inventories = app.ecs.write_storage::<Inventory>();
    let item_name = names.get(item_entity.unwrap()).expect("Unable to access name item name during trading").name.clone();
    let item = items.get(item_entity.unwrap()).expect("Unable to access item component during trading");
    let player_inventory = inventories.get_mut(*player_entity).expect("Unable to access player inventory during trading");
    let vendor = vendors.get_mut(vendor_entity).expect("Unable to access vendor component during trading");

    match is_buying {
        true => {
            if player_inventory.gold >= item.base_value {
                info!(
                    "Purchasing item {} from vendor, item costs {} and player has {} gold",
                    item_name,
                    item.base_value,
                    player_inventory.gold,
                );
                player_inventory.gold -= item.base_value;
                pickups.insert(*player_entity, WantsToPickupItem {
                    collected_by: *player_entity,
                    items: vec![item_entity.unwrap()],
                }).expect("uhhh");
                vendor.items.remove(vendor_index);
                app.screen = Screen::Trading {
                    vendor: vendor_entity,
                    vendor_index: 0,
                    player_index: player_index,
                    is_buying: is_buying,
                };
                Logger::new()
                    .append("You buy the ")
                    .append_with_color(Color::Blue, item_name)
                    .append(" for ")
                    .append_with_color(Color::Yellow, format!("{} gold.", item.base_value))
                    .log();
            } else {
                Logger::new()
                    .append("You don't have enough gold to buy the ")
                    .append_with_color(Color::Blue, format!("{}.", item_name))
                    .log();
            }
        }

        false => {
            info!(
                "Selling item {} to vendor for {} gold",
                item_name,
                item.base_value,
            );
            player_inventory.gold += item.base_value;
            player_inventory.index = 0;
            match player_inventory.items.entry(item_name.clone()) {
                indexmap::map::Entry::Occupied(mut entry) => {
                    let stack = entry.get_mut();
                    let _ = stack.remove(0);
                    if stack.is_empty() {
                        entry.shift_remove();
                    }
                }
                indexmap::map::Entry::Vacant(_) => {}
            }
            let item_base_value = item.base_value;
            items.remove(item_entity.unwrap());
            app.screen = Screen::Trading {
                vendor: vendor_entity,
                vendor_index: vendor_index,
                player_index: 0,
                is_buying: is_buying,
            };
            Logger::new()
                .append("You sell the ")
                .append_with_color(Color::Blue, format!("{} ", item_name))
                .append("for ")
                .append_with_color(Color::Yellow, format!("{} gold.", item_base_value))
                .log();
        }
    }
    return None;
}
