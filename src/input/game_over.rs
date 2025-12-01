use crossterm::event::{KeyCode, KeyEvent};

use crate::{floor::generate_floor, reinitialize_systems, reinitialize_world, App, RootScreen};

pub fn handle_game_over_key_event(app: &mut App, key_event: KeyEvent) -> bool {
    match key_event.code {
        KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Esc => {
            app.ecs = reinitialize_world();
            app.dispatcher = reinitialize_systems(&mut app.ecs);
            generate_floor(0, 0, &mut app.ecs);
            app.root_screen = RootScreen::Menu;
            return false;
        }
        _ => false,
    }
}
