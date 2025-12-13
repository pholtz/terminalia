use crossterm::event::{KeyCode, KeyEvent};

use crate::{App, RootScreen, RunState, Screen};

pub fn handle_main_quit_key_event(app: &mut App, quit: bool, key_event: KeyEvent) -> Option<RunState> {
    match key_event.code {
        KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('h') => {
            app.screen = Screen::Quit { quit: !quit };
            return None;
        }
        KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('l') => {
            app.screen = Screen::Quit { quit: !quit };
            return None;
        }
        KeyCode::Enter => {
            if quit {
                app.root_screen = RootScreen::Menu;
            } else {
                app.screen = Screen::Explore;
            }
            return None;
        }
        _ => None,
    }
}
