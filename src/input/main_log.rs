use std::sync::atomic::Ordering;

use ratatui::style::Color;
use specs::prelude::*;

use crossterm::{event::{KeyCode, KeyEvent}};

use crate::{
    App, RunState, Screen, component::Stats, logbook::logbook::{self, LOG_INDEX, Logger}
};

pub fn handle_main_log_key_event(app: &mut App, key_event: KeyEvent) -> Option<RunState> {
    /*
     * Handle user input into logbook cases
     */
    let key_char = key_event.code.as_char();
    if app.log_index == 1 {
        if key_char
            .filter(|c| c.is_alphanumeric() || [' ', '/'].contains(c))
            .is_some()
        {
            app.logbook_input.push(key_char.unwrap_or('?'));
            return None;
        }

        if key_event.code == KeyCode::Backspace {
            app.logbook_input.pop();
            return None;
        }

        if key_event.code == KeyCode::Enter && !app.logbook_input.is_empty() {
            Logger::new()
                .append(format!("> {}", app.logbook_input))
                .log();
            if app.logbook_input.starts_with("/") {
                process_command(app.logbook_input.clone(), &mut app.ecs);
            }
            app.logbook_input = "".to_string();
            return None;
        }
    }

    /*
     * Handle all reader view and other fallback inputs.
     */
    match key_event.code {
        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
            let _ = LOG_INDEX
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |index| {
                    if index == 0 { None } else { Some(index - 1) }
                })
                .is_ok();
            return None;
        }

        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
            let _ = LOG_INDEX
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |index| {
                    let logbook_size: u16 = logbook::size()
                        .try_into()
                        .expect("Unable to convert logbook size from usize -> u16");
                    if index + 1 >= logbook_size {
                        None
                    } else {
                        Some(index + 1)
                    }
                })
                .is_ok();
            return None;
        }

        KeyCode::Tab => {
            app.log_index = match app.log_index {
                0 => 1,
                1 => 0,
                _ => 0,
            };
            return None;
        }

        KeyCode::Esc => {
            app.screen = Screen::Explore;
            return None;
        }
        _ => None,
    }
}

pub fn process_command(input: String, ecs: &mut World) {
    if input.starts_with("/health") {
        let player_entity = ecs.read_resource::<Entity>();
        let mut stats = ecs.write_storage::<Stats>();
        if let Some(stat) = stats.get_mut(*player_entity) {
            stat.hp.current = stat.hp.max;
            Logger::new().append_with_color(Color::Yellow, "You were healed!").log();
        }
    }
}
