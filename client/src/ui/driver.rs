use std::{
    io::{self, Stdout},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use secrecy::SecretSlice;

use crate::{core, error::AppError, transport::error::TransportError};

use super::{
    app::App,
    draw,
    view::{FormKind, LoginField, MenuPageKind, MenuState, SignupField, View},
};

pub fn run(app: App) -> io::Result<()> {
    // Start terminal in raw mode and alternate screen then run the UI loop
    let mut terminal = init_terminal()?;
    let mut app = app;
    let res = run_app(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;
    res
}

// --- Terminal setup/restore ---

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    // Raw mode for direct key input and alternate screen for a clean full screen UI
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    // Always clean up so the user terminal is restored on exit
    crossterm::terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

// --- Runtime loop ---

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> io::Result<()> {
    // Basic tick loop draw poll for input update timers
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        // Advance timers like menu status rotation
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

        if last_tick.elapsed() >= tick_rate {
            // Time based updates can hook into this later
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
            View::Form(form) => ExitAction::ReturnToMenu(form.return_menu.clone()),
            View::Info(info) => ExitAction::ReturnToMenu(info.return_menu.clone()),
            View::Chat(_) => {
                let menu = if app.session.is_some() {
                    MenuState::logged_in()
                } else {
                    MenuState::logged_out()
                };
                ExitAction::ReturnToMenu(menu)
            }
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
        View::Form(form) => match &form.kind {
            FormKind::Login(_) => handle_login_key,
            FormKind::Signup(_) => handle_signup_key,
        },
        View::Info(_) => handle_info_key,
        View::Chat(_) => handle_chat_key,
    };
    handler(app, key);

    app.should_quit
}

fn handle_menu_key(app: &mut App, key: KeyEvent) {
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

fn handle_info_key(_app: &mut App, _key: KeyEvent) {}

fn handle_chat_key(_app: &mut App, _key: KeyEvent) {}

fn handle_login_key(app: &mut App, key: KeyEvent) {
    // Login form input username then password then submit
    let mut action = LoginAction::None;

    {
        let form = match &mut app.view {
            View::Form(form) => form,
            _ => return,
        };
        let return_menu = form.return_menu.clone();
        let error = &mut form.error;
        let login = match &mut form.kind {
            FormKind::Login(login) => login,
            _ => return,
        };

        match key.code {
            KeyCode::Esc => {
                action = LoginAction::Back(return_menu);
            }
            KeyCode::Enter => {
                *error = None;
                match login.active {
                    LoginField::Username => {
                        if login.username.trim().is_empty() {
                            *error = Some("Username required.".to_string());
                        } else {
                            login.active = LoginField::Password;
                        }
                    }
                    LoginField::Password => {
                        let username = login.username.trim();
                        if username.is_empty() {
                            *error = Some("Username required.".to_string());
                            login.active = LoginField::Username;
                        } else if login.password_is_empty() {
                            *error = Some("Password required.".to_string());
                        } else {
                            action = LoginAction::Submit {
                                username: username.to_string(),
                                password: login.take_password(),
                            };
                        }
                    }
                }
            }
            KeyCode::Backspace => match login.active {
                LoginField::Username => {
                    login.username.pop();
                }
                LoginField::Password => {
                    login.pop_password_char();
                }
            },
            KeyCode::Char(ch) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    match login.active {
                        LoginField::Username => login.username.push(ch),
                        LoginField::Password => login.push_password_char(ch),
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
        LoginAction::Submit { username, password } => {
            // Submit credentials to core auth and update UI state based on result
            match core::auth::perform_login(&username, &password) {
                Ok(session) => {
                    app.session = Some(session);
                    app.view = View::Menu(MenuState::logged_in());
                }
                Err(err) => {
                    // Keep friendly errors for invalid credentials fallback to raw error text
                    let message = match err {
                        AppError::Transport(TransportError::LoginFailed)
                        | AppError::Transport(TransportError::Unauthorized) => {
                            "Username or password is incorrect.".to_string()
                        }
                        _ => err.to_string(),
                    };
                    if let View::Form(form) = &mut app.view {
                        form.error = Some(message);
                        if let FormKind::Login(login) = &mut form.kind {
                            // Security and UX clear password and reset focus to the first field
                            login.clear_password();
                            login.active = LoginField::Username;
                        }
                    }
                }
            }
        }
    }
}

enum LoginAction {
    None,
    Back(MenuState),
    Submit {
        username: String,
        password: SecretSlice<u8>,
    },
}

fn handle_signup_key(app: &mut App, key: KeyEvent) {
    // Signup form input username then password then confirm password then submit
    let mut action = SignupAction::None;

    {
        let form = match &mut app.view {
            View::Form(form) => form,
            _ => return,
        };
        let return_menu = form.return_menu.clone();
        let error = &mut form.error;
        let signup = match &mut form.kind {
            FormKind::Signup(signup) => signup,
            _ => return,
        };

        match key.code {
            KeyCode::Esc => {
                action = SignupAction::Back(return_menu);
            }
            KeyCode::Enter => {
                *error = None;
                match signup.active {
                    SignupField::Username => {
                        if signup.username.trim().is_empty() {
                            *error = Some("Username required.".to_string());
                        } else {
                            signup.active = SignupField::Password;
                        }
                    }
                    SignupField::Password => {
                        if signup.password_is_empty() {
                            *error = Some("Password required.".to_string());
                        } else {
                            signup.active = SignupField::ConfirmPassword;
                        }
                    }
                    SignupField::ConfirmPassword => {
                        if signup.confirm_is_empty() {
                            *error = Some("Confirm password required.".to_string());
                        } else if !signup.passwords_match() {
                            *error = Some("Passwords do not match.".to_string());
                            signup.clear_passwords();
                            signup.active = SignupField::Password;
                        } else {
                            action = SignupAction::Submit {
                                username: signup.username.trim().to_string(),
                                password: signup.take_password(),
                            };
                        }
                    }
                }
            }
            KeyCode::Backspace => match signup.active {
                SignupField::Username => {
                    signup.username.pop();
                }
                SignupField::Password => {
                    signup.pop_password_char();
                }
                SignupField::ConfirmPassword => {
                    signup.pop_confirm_char();
                }
            },
            KeyCode::Char(ch) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    match signup.active {
                        SignupField::Username => signup.username.push(ch),
                        SignupField::Password => signup.push_password_char(ch),
                        SignupField::ConfirmPassword => signup.push_confirm_char(ch),
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
        SignupAction::Submit { username, password } => {
            match core::auth::perform_signup(&username, &password) {
                Ok(_) => {
                    // TODO REMOVE :
                    app.view = View::Menu(MenuState::logged_out());
                }
                Err(err) => {
                    // TODO : Handle signup error
                    Ok(_) => state.set_action_msg("Registration successful!"),
                    state.set_action_msg(format!("Registration failed: {e}")),
                }
            }
        }
    }
}

enum SignupAction {
    None,
    Back(MenuState),
    Submit {
        username: String,
        password: SecretSlice<u8>,
    },
}
