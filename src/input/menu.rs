use crossterm::event::{KeyCode, KeyEvent};
use rand::Rng;

use crate::{App, RootScreen, RunState, Screen, generate::generator::generate_floor, logbook::{logbook::{self, Logger}}, reinitialize_world};

pub fn handle_menu_key_event(app: &mut App, key_event: KeyEvent) -> Option<RunState> {
    match key_event.code {
        KeyCode::Esc => app.exit(),
        KeyCode::Up | KeyCode::Char('w') => {
            if app.menu_index == 0 {
                app.menu_index = 1;
            } else {
                app.menu_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('s') => {
            if app.menu_index == 1 {
                app.menu_index = 0;
            } else {
                app.menu_index += 1;
            }
        }
        KeyCode::Enter => match app.menu_index {
            0 => {
                app.ecs = reinitialize_world();
                generate_floor(rand::rng().random(), 0, &mut app.ecs);
                app.root_screen = RootScreen::Main;
                app.screen = Screen::Explore;
                logbook::clear();
                Logger::new().append("You begin your adventure in a smallish room...").log();
            }
            1 => app.exit(),
            _ => {}
        },
        _ => {}
    }
    return None
}
