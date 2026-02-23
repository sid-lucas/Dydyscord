use crossterm::event::{KeyCode, KeyEvent};

use crate::ui::{app::App, view::View};

pub fn handle_menu_key(app: &mut App, key: KeyEvent) {
    // Menu input move selection enter or go back
    let kind = match &app.view {
        View::Menu(menu) => menu.current().kind,
        _ => return,
    };
    let entries = app.menu_entries(kind);
    let mut activate = false;

    {
        let menu = match &mut app.view {
            View::Menu(menu) => menu,
            _ => return,
        };
        menu.current_mut().clamp(entries.len());

        match key.code {
            KeyCode::Up => menu.current_mut().move_selection(-1, entries.len()),
            KeyCode::Down => menu.current_mut().move_selection(1, entries.len()),
            KeyCode::PageUp => menu.current_mut().move_selection(-5, entries.len()),
            KeyCode::PageDown => menu.current_mut().move_selection(5, entries.len()),
            KeyCode::Enter => activate = true,
            KeyCode::Esc | KeyCode::Backspace => {
                // Back to previous menu at root it stays put
                menu.pop();
            }
            _ => {}
        }
    }

    if activate {
        app.activate_menu_selection();
    }
}
