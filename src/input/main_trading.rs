use specs::prelude::*;
use crossterm::event::{KeyCode, KeyEvent};

use crate::{App, RunState, Screen};

pub fn handle_main_trading_key_event(
    app: &mut App,
    key_event: KeyEvent,
    vendor: Entity,
    index: usize,
) -> Option<RunState> {
    match key_event.code {
        KeyCode::Up => {
            app.screen = Screen::Trading {
                vendor: vendor,
                index: index - 1,
            };
            None
        }

        KeyCode::Down => {
            app.screen = Screen::Trading {
                vendor: vendor,
                index: index + 1,
            };
            None
        }

        KeyCode::Enter | KeyCode::Char(' ') => {
            None
        }

        KeyCode::Esc => {
            app.screen = Screen::Explore;
            None
        }

        _ => None
    }
}