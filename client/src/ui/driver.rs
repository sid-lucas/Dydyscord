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

use super::{
    app::App,
    draw,
    view::{LoginField, MenuPageKind, MenuState, SignupField, View},
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
    let is_ctrl_c = key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);
    let is_esc = key.code == KeyCode::Esc;

    if is_ctrl_c || is_esc {
        enum ExitAction {
            Quit,
            Logout,
            MenuBack,
            ReturnToMenu(MenuState),
        }

        let action = match &app.view {
            View::Menu(menu) => {
                if menu.stack.len() > 1 {
                    ExitAction::MenuBack
                } else if menu.current().kind == MenuPageKind::LoggedOut {
                    ExitAction::Quit
                } else {
                    ExitAction::Logout
                }
            }
            View::Login(form) => ExitAction::ReturnToMenu(form.return_menu.clone()),
            View::Signup(form) => ExitAction::ReturnToMenu(form.return_menu.clone()),
        };

        match action {
            ExitAction::Quit => {
                app.should_quit = true;
                return true;
            }
            ExitAction::Logout => {
                app.logout();
                return app.should_quit;
            }
            ExitAction::MenuBack => {
                if let View::Menu(menu) = &mut app.view {
                    menu.pop();
                }
                return app.should_quit;
            }
            ExitAction::ReturnToMenu(menu) => {
                app.view = View::Menu(menu);
                return app.should_quit;
            }
        }
    }

    let handler: fn(&mut App, KeyEvent) = match &app.view {
        View::Menu(_) => handle_menu_key,
        View::Login(_) => handle_login_key,
        View::Signup(_) => handle_signup_key,
    };
    handler(app, key);

    app.should_quit
}

fn handle_menu_key(app: &mut App, key: KeyEvent) {
    // Menu input: move selection, enter, or go back.
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
                // Back to previous menu; at root it just stays put.
                menu.pop();
            }
            _ => {}
        }
    }

    if activate {
        app.activate_menu_selection();
    }
}

fn handle_login_key(app: &mut App, key: KeyEvent) {
    // Login form input: username -> password -> submit.
    let mut action = LoginAction::None;

    {
        let form = match &mut app.view {
            View::Login(form) => form,
            _ => return,
        };

        match key.code {
            KeyCode::Esc => {
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
                            action = LoginAction::Success {
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
            app.view = View::Menu(menu);
        }
        LoginAction::Success { username, password } => {
            app.session = None; //Some(Session::new(username));
            app.view = View::Menu(MenuState::logged_in());
        }
    }
}

enum LoginAction {
    None,
    Back(MenuState),
    Success {
        username: String,
        password: SecretSlice<u8>,
    },
}

fn handle_signup_key(app: &mut App, key: KeyEvent) {
    // Signup form input: username -> password -> confirm password -> submit.
    let mut action = SignupAction::None;

    {
        let form = match &mut app.view {
            View::Signup(form) => form,
            _ => return,
        };

        match key.code {
            KeyCode::Esc => {
                action = SignupAction::Back(form.return_menu.clone());
            }
            KeyCode::Enter => {
                form.error = None;
                match form.active {
                    SignupField::Username => {
                        if form.username.trim().is_empty() {
                            form.error = Some("Username required.".to_string());
                        } else {
                            form.active = SignupField::Password;
                        }
                    }
                    SignupField::Password => {
                        if form.password_is_empty() {
                            form.error = Some("Password required.".to_string());
                        } else {
                            form.active = SignupField::ConfirmPassword;
                        }
                    }
                    SignupField::ConfirmPassword => {
                        if form.confirm_is_empty() {
                            form.error = Some("Confirm password required.".to_string());
                        } else if !form.passwords_match() {
                            form.error = Some("Passwords do not match.".to_string());
                            form.clear_passwords();
                            form.active = SignupField::Password;
                        } else {
                            action = SignupAction::Success {
                                username: form.username.trim().to_string(),
                                password: form.take_password(),
                            };
                        }
                    }
                }
            }
            KeyCode::Backspace => match form.active {
                SignupField::Username => {
                    form.username.pop();
                }
                SignupField::Password => {
                    form.pop_password_char();
                }
                SignupField::ConfirmPassword => {
                    form.pop_confirm_char();
                }
            },
            KeyCode::Char(ch) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    match form.active {
                        SignupField::Username => form.username.push(ch),
                        SignupField::Password => form.push_password_char(ch),
                        SignupField::ConfirmPassword => form.push_confirm_char(ch),
                    }
                }
            }
            _ => {}
        }
    }

    match action {
        SignupAction::None => {}
        SignupAction::Back(menu) => {
            app.view = View::Menu(menu);
        }
        SignupAction::Success { username, password } => {
            app.session = None; //Some(Session::new(username));
            app.view = View::Menu(MenuState::logged_out());
        }
    }
}

enum SignupAction {
    None,
    Back(MenuState),
    Success {
        username: String,
        password: SecretSlice<u8>,
    },
}
