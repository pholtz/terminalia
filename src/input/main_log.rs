use std::sync::atomic::Ordering;

use crossterm::event::{KeyCode, KeyEvent};

use crate::{App, RunState, Screen, logbook::logbook::{self, LOG_INDEX}};

pub fn handle_main_log_key_event(app: &mut App, key_event: KeyEvent) -> Option<RunState> {
    match key_event.code {
        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
            let _ = LOG_INDEX.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |index| {
                if index == 0 { None } else { Some(index - 1) }
            }).is_ok();
            return None;
        }

        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
            let _ = LOG_INDEX.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |index| {
                let logbook_size: u16 = logbook::size().try_into().expect("Unable to convert logbook size from usize -> u16");
                if index + 1 >= logbook_size { None } else { Some(index + 1) }
            }).is_ok();
            return None;
        }

        KeyCode::Char('q') | KeyCode::Esc => {
            app.screen = Screen::Explore;
            return None;
        }
        _ => None,
    }
}
