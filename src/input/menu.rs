use crossterm::event::{KeyCode, KeyEvent};

use crate::{floor::generate_floor, reinitialize_world, App, RootScreen};

pub fn handle_menu_key_event(app: &mut App, key_event: KeyEvent) -> bool {
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
                generate_floor(0, 0, &mut app.ecs);
                app.root_screen = RootScreen::Main;
            }
            1 => app.exit(),
            _ => {}
        },
        _ => {}
    }
    return false
}
