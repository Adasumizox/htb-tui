use crate::app::{App, AppResult, InputMode};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_key_events(key_event: KeyEvent, app: &mut App) ->AppResult<()> {
    match app.input_mode {
        InputMode::Normal => match key_event.code {
            KeyCode::Char('q') => app.quit(),
            KeyCode::Char('f') => app.cycle_filter(),
            KeyCode::Char('s') => app.cycle_sort(),
            KeyCode::Down => app.next(),
            KeyCode::Up => app.previous(),
            KeyCode::Char('a') => app.enter_flag_input_mode(),
            KeyCode::Enter => {
                todo!();
            }
            _ => {}
        },
        InputMode::Flag => todo!(),
    }
    Ok(())
}
