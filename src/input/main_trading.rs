use log::info;
use ratatui::style::Color;
use specs::prelude::*;
use crossterm::event::{KeyCode, KeyEvent};

use crate::{App, RunState, Screen, component::{Inventory, Item, Name, Vendor, WantsToPickupItem}, logbook::logbook::Logger};

pub fn handle_main_trading_key_event(
    app: &mut App,
    key_event: KeyEvent,
    vendor: Entity,
    index: usize,
) -> Option<RunState> {
    match key_event.code {
        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
            if index > 0 {
                app.screen = Screen::Trading {
                    vendor: vendor,
                    index: index - 1,
                };
            }
            None
        }

        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
            let vendors = app.ecs.read_storage::<Vendor>();
            let v = vendors.get(vendor).expect("Unable to access given vendor");
            if index + 1 < v.items.len() {
                app.screen = Screen::Trading {
                    vendor: vendor,
                    index: index + 1,
                };
            }
            None
        }

        /*
         * Attempt to buy the currently selected item
         */
        KeyCode::Enter | KeyCode::Char(' ') => {
            try_buy_item(app, vendor, index)
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
fn try_buy_item(app: &mut App, vendor_entity: Entity, index: usize) -> Option<RunState> {
    let names = app.ecs.read_storage::<Name>();
    let items = app.ecs.read_storage::<Item>();
    let mut vendors = app.ecs.write_storage::<Vendor>();
    let mut inventories = app.ecs.write_storage::<Inventory>();
    let mut pickups = app.ecs.write_storage::<WantsToPickupItem>();
    let player_entity = app.ecs.fetch::<Entity>();

    let player_inventory = inventories.get_mut(*player_entity).expect("Unable to access player inventory during trading");
    let vendor = vendors.get_mut(vendor_entity).expect("Unable to access vendor component during trading");
    let item_entity = vendor.items[index];
    let item_name = names.get(item_entity).expect("Unable to access name item component during trading").name.clone();
    let item = items.get(item_entity).expect("Unable to access vendor item component during trading");

    if player_inventory.gold >= item.base_value {
        info!(
            "Purchasing item {} from vendor, item costs {} and player has {} gold",
            item.description,
            item.base_value,
            player_inventory.gold,
        );
        player_inventory.gold -= item.base_value;
        pickups.insert(*player_entity, WantsToPickupItem {
            collected_by: *player_entity,
            items: vec![item_entity],
        }).expect("uhhh");
        vendor.items.remove(index);
        app.screen = Screen::Trading { vendor: vendor_entity, index: 0 };
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
    return None;
}
