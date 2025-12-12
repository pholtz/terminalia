use crossterm::event::{KeyCode, KeyEvent};
use specs::prelude::*;

use crate::{App, RunState, Screen};

pub fn handle_main_log_key_event(app: &mut App, key_event: KeyEvent) -> Option<RunState> {
    match key_event.code {
        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
            try_scroll_logbook(&mut app.ecs, -1)
        }

        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
            try_scroll_logbook(&mut app.ecs, 1)
        }

        KeyCode::Char('q') | KeyCode::Esc => {
            app.screen = Screen::Explore;
            return None;
        }
        _ => None,
    }
}

fn try_scroll_logbook(ecs: &mut World, delta: i16) -> Option<RunState> {
    // if delta.is_positive() {
    //     match logbook.scroll_offset.checked_add(delta as u16) {
    //         Some(offset) => {
    //             if offset <= ((logbook.entries.len() - 1) as u16) {
    //                 logbook.scroll_offset = offset
    //             }
    //         }
    //         None => {}
    //     }
    // } else {
    //     match logbook.scroll_offset.checked_sub(delta.abs() as u16) {
    //         Some(offset) => logbook.scroll_offset = offset,
    //         None => {}
    //     }
    // }
    return None;
}
