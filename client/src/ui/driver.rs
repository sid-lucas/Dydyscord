use std::{
    io::{self, Stdout},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use super::{
    app::{App, MenuState, View},
    draw,
};

pub fn run(app: App) -> io::Result<()> {
    // Boot terminal in raw mode + alternate screen, then run the UI loop.
    let mut terminal = init_terminal()?;
    let mut app = app;
    let res = run_app(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;
    res
}

// --- Terminal setup/restore ---

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    // Raw mode = direct key input, alternate screen = clean full-screen UI.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    // Always clean up so the user's terminal is restored on exit.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

// --- Runtime loop ---

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> io::Result<()> {
    // Basic tick loop: draw, poll for input, update timers.
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        // Advance timers (menu status rotation, etc.).
        app.tick();
        // Draw the current frame.
        terminal.draw(|f| draw::ui(f, app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Return true to quit.
                if handle_key(app, key) {
                    return Ok(()); // quit
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            // Time-based updates can hook into this if needed later.
            last_tick = Instant::now();
        }
    }
}

// --- Event handling ---

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    // Global quit shortcut.
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return true;
    }

    // Snapshot the view so we can borrow app safely in handlers.
    let view_snapshot = app.view.clone();
    match view_snapshot {
        View::Menu(_) => handle_menu_key(app, key),
        View::Chat { room_index } => handle_chat_key(app, room_index, key),
        View::Info(_) => handle_info_key(app, key),
    };

    app.should_quit
}

fn handle_menu_key(app: &mut App, key: KeyEvent) {
    // Menu input: move selection, enter, or go back.
    let kind = match &app.view {
        View::Menu(menu) => menu.current().kind,
        _ => return,
    };
    let entries = app.menu_entries(kind);
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
        KeyCode::Enter => app.activate_menu_selection(),
        KeyCode::Esc | KeyCode::Backspace => {
            // Back to previous menu, or quit if at root.
            if !menu.pop() {
                app.should_quit = true;
            }
        }
        _ => {}
    }
}

fn handle_info_key(app: &mut App, key: KeyEvent) {
    // Info screen only needs a "back" action.
    let info = match &mut app.view {
        View::Info(info) => info,
        _ => return,
    };

    if matches!(key.code, KeyCode::Esc | KeyCode::Backspace | KeyCode::Enter) {
        // Restore the previous menu snapshot.
        app.view = View::Menu(info.return_menu.clone());
    }
}

fn handle_chat_key(app: &mut App, room_index: usize, key: KeyEvent) {
    // Chat input: Esc goes back to chatrooms list.
    if key.code == KeyCode::Esc {
        app.back_to_chatrooms(room_index);
        return;
    }

    // Find the current room; fall back to root menu if it's missing.
    let chat = match app.rooms.get_mut(room_index) {
        Some(room) => &mut room.chat,
        None => {
            app.view = View::Menu(MenuState::root());
            return;
        }
    };

    match key.code {
        KeyCode::Enter => {
            // Send message (and clear input).
            let msg = std::mem::take(&mut chat.input);
            chat.push_message(chat.user_name.clone(), msg);
        }
        KeyCode::Backspace => {
            // Remove last character.
            chat.input.pop();
        }
        KeyCode::Char(ch) => {
            // Ignore CTRL combos; only collect text input.
            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                chat.input.push(ch);
            }
        }

        KeyCode::Up => chat.scroll_up(),
        KeyCode::Down => chat.scroll_down(),
        KeyCode::PageUp => chat.page_up(),
        KeyCode::PageDown => chat.page_down(),

        _ => {}
    }
}
