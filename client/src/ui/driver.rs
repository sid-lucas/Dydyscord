use std::{
    io::{self, Stdout},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use secrecy::SecretSlice;

use crate::{core, transport, ui::flows};
use crate::auth::session::Session;

use super::{
    app::{App, BlockingAction, LoginField, MenuPageKind, MenuState, View},
    draw,
};

// Entry point for the ratatui UI loop
pub fn run(app: App) -> io::Result<()> {
    // Boot terminal in raw mode + alternate screen, then run the UI loop
    let mut terminal = init_terminal()?;
    let mut app = app;
    let res = run_app(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;
    res
}

// --- Terminal setup/restore ---

// Configure the terminal for raw mode and alternate screen
fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    // Raw mode = direct key input, alternate screen = clean full-screen UI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

// Restore terminal settings on exit
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    // Always clean up so the user's terminal is restored on exit
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

// --- Runtime loop ---

// Main UI loop that draws frames and handles input
fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> io::Result<()> {
    // Basic tick loop: draw, poll for input, update timers
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        // Advance timers (menu status rotation, etc.)
        app.tick();
        // Draw the current frame
        terminal.draw(|f| draw::ui(f, app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Return true to quit
                if handle_key(app, key) {
                    return Ok(()); // quit
                }
            }
        }

        if let Some(action) = app.runtime.pending_action.take() {
            run_blocking_action(terminal, app, action)?;
            last_tick = Instant::now();
        }

        if last_tick.elapsed() >= tick_rate {
            // Time-based updates can hook into this if needed later
            last_tick = Instant::now();
        }
    }
}

// --- Event handling ---

// Route input based on the current view
fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    // Snapshot the view so we can borrow app safely in handlers
    let view_snapshot = app.ui.view.clone();
    match view_snapshot {
        View::Menu(_) => handle_menu_key(app, key),
        View::Chat { room_index } => handle_chat_key(app, room_index, key),
        View::Info(_) => handle_info_key(app, key),
        View::Login(_) => handle_login_key(app, key),
    };

    app.runtime.should_quit
}

// Handle input when a menu is active
fn handle_menu_key(app: &mut App, key: KeyEvent) {
    // Menu input: move selection, enter, or go back
    let kind = match &app.ui.view {
        View::Menu(menu) => menu.current().kind,
        _ => return,
    };
    let entries = app.menu_entries(kind);
    let is_ctrl_c = key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);
    let is_esc = key.code == KeyCode::Esc;
    let is_back = is_ctrl_c || is_esc;
    let mut activate = false;
    let mut nav_action = MenuNavAction::None;

    {
        let menu = match &mut app.ui.view {
            View::Menu(menu) => menu,
            _ => return,
        };
        menu.current_mut().clamp(entries.len());

        if is_back {
            nav_action = match menu.current().kind {
                MenuPageKind::RootGuest => MenuNavAction::Quit,
                MenuPageKind::RootAuthed => MenuNavAction::Logout,
                _ => MenuNavAction::Pop,
            };
        } else {
            match key.code {
                KeyCode::Up => menu.current_mut().move_selection(-1, entries.len()),
                KeyCode::Down => menu.current_mut().move_selection(1, entries.len()),
                KeyCode::PageUp => menu.current_mut().move_selection(-5, entries.len()),
                KeyCode::PageDown => menu.current_mut().move_selection(5, entries.len()),
                KeyCode::Enter => activate = true,
                _ => {}
            }
        }
    }

    if activate {
        app.activate_menu_selection();
    }

    match nav_action {
        MenuNavAction::None => {}
        MenuNavAction::Pop => {
            if let View::Menu(menu) = &mut app.ui.view {
                menu.pop();
            }
        }
        MenuNavAction::Logout => {
            app.logout();
        }
        MenuNavAction::Quit => {
            app.runtime.should_quit = true;
        }
    }
}

// Handle input on info screens
fn handle_info_key(app: &mut App, key: KeyEvent) {
    // Info screen only needs a "back" action
    let info = match &mut app.ui.view {
        View::Info(info) => info,
        _ => return,
    };

    let is_ctrl_c = key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);

    if key.code == KeyCode::Esc || is_ctrl_c {
        // Restore the previous menu snapshot
        app.ui.view = View::Menu(info.return_menu.clone());
    }
}

// Handle input while inside a chatroom
fn handle_chat_key(app: &mut App, room_index: usize, key: KeyEvent) {
    // Chat input: Esc goes back to chatrooms list
    let is_ctrl_c = key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);

    if key.code == KeyCode::Esc || is_ctrl_c {
        app.back_to_chatrooms(room_index);
        return;
    }

    // Find the current room; fall back to root menu if it's missing
    let chat = match app
        .data
        .rooms
        .as_mut()
        .and_then(|rooms| rooms.get_mut(room_index))
    {
        Some(room) => &mut room.chat,
        None => {
            app.ui.view = View::Menu(MenuState::root());
            return;
        }
    };

    match key.code {
        KeyCode::Enter => {
            // Send message (and clear input)
            let msg = std::mem::take(&mut chat.input);
            chat.push_message(chat.user_name.clone(), msg);
        }
        KeyCode::Backspace => {
            // Remove last character
            chat.input.pop();
        }
        KeyCode::Char(ch) => {
            // Ignore CTRL combos; only collect text input
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

// Handle input for the login form
fn handle_login_key(app: &mut App, key: KeyEvent) {
    // Login form input: username -> password -> submit
    let mut action = LoginAction::None;

    {
        let form = match &mut app.ui.view {
            View::Login(form) => form,
            _ => return,
        };
        let is_ctrl_c =
            key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Esc => {
                action = LoginAction::Back(form.return_menu.clone());
            }
            _ if is_ctrl_c => {
                action = LoginAction::Back(form.return_menu.clone());
            }
            KeyCode::Enter => {
                form.error = None;
                match form.active {
                    LoginField::Username => {
                        if form.username.trim().is_empty() {
                            form.error = Some("Username required.".to_string());
                        } else {
                            form.active = LoginField::Password;
                        }
                    }
                    LoginField::Password => {
                        let username = form.username.trim();
                        if username.is_empty() {
                            form.error = Some("Username required.".to_string());
                            form.active = LoginField::Username;
                        } else if form.password_is_empty() {
                            form.error = Some("Password required.".to_string());
                        } else {
                            action = LoginAction::Submit {
                                username: username.to_string(),
                                password: form.take_password(),
                            };
                        }
                    }
                }
            }
            KeyCode::Backspace => match form.active {
                LoginField::Username => {
                    form.username.pop();
                }
                LoginField::Password => {
                    form.pop_password_char();
                }
            },
            KeyCode::Char(ch) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    match form.active {
                        LoginField::Username => form.username.push(ch),
                        LoginField::Password => form.push_password_char(ch),
                    }
                }
            }
            _ => {}
        }
    }

    match action {
        LoginAction::None => {}
        LoginAction::Back(menu) => {
            app.ui.view = View::Menu(menu);
        }
        LoginAction::Submit { username, password } => {
            match perform_login(&username, &password) {
                Ok(session) => {
                    app.set_session(Some(session));
                    app.ui.view = View::Menu(MenuState::authed_root());
                    transport::ws::start_background();
                }
                Err(msg) => {
                    if let View::Login(form) = &mut app.ui.view {
                        form.error = Some(msg);
                        form.clear_password();
                    }
                }
            }
        }
    }
}

// Menu-level navigation intent produced by input
enum MenuNavAction {
    None,
    Pop,
    Logout,
    Quit,
}

// Login flow intent produced by input
enum LoginAction {
    None,
    Back(MenuState),
    Submit { username: String, password: SecretSlice<u8> },
}

// Execute the real login flow and return a populated session
fn perform_login(username: &str, password: &SecretSlice<u8>) -> Result<Session, String> {
    core::auth::login(username, password).map_err(|e| format!("Login failed: {e}"))
}

// Temporarily exit ratatui to run blocking CLI flows
fn run_blocking_action(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    action: BlockingAction,
) -> io::Result<()> {
    restore_terminal(terminal)?;
    flows::run(app, action);
    *terminal = init_terminal()?;
    Ok(())
}
