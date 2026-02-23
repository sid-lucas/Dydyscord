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

use crate::ui::{
    app::App,
    chat::driver as chat_driver,
    draw,
    form::view::FormKind,
    info::driver as info_driver,
    menu::view::{MenuPageKind, MenuState},
    view::View,
};

use crate::ui::{form::driver as form_driver, menu::driver as menu_driver};

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
        View::Menu(_) => menu_driver::handle_menu_key,
        View::Form(form) => match &form.kind {
            FormKind::Login(_) => form_driver::handle_login_key,
            FormKind::Signup(_) => form_driver::handle_signup_key,
            FormKind::GroupCreate(_) => form_driver::handle_groupcreate_key,
        },
        View::Info(_) => info_driver::handle_info_key,
        View::Chat(_) => chat_driver::handle_chat_key,
    };
    handler(app, key);

    app.should_quit
}
