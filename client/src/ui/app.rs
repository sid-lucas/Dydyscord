use std::time::{Duration, Instant};

use crate::{auth::session::Session, config::constant};

use super::view::{
    LoginFormState, MenuAction, MenuEntry, MenuPageKind, MenuState, SignupFormState, View,
};

// How often the menu status line rotates.
const STATUS_ROTATE_EVERY: Duration = Duration::from_secs(2);
// Rotating messages shown only in menu views.
const STATUS_MESSAGES: &[&str] = &[
    "Etat: synchronisation des salons...",
    "Tip: utilisez PgUp/PgDn pour naviguer plus vite.",
    "Actu: nouveaux salons bientot disponibles.",
    "Info: vos messages sont sauvegardes localement (placeholder).",
    "Astuce: Esc remonte d'un niveau de menu.",
];

pub struct App {
    pub name: String,
    pub version: String,
    pub session: Option<Session>,
    // Which screen is currently active.
    pub view: View,
    // Signal to stop the app loop.
    pub should_quit: bool,
    // Cached status message so it stays stable across menu navigation.
    menu_status: String,
    // Index into STATUS_MESSAGES.
    menu_status_index: usize,
    // Last time the status was updated.
    menu_status_updated_at: Instant,
}

impl App {
    pub fn new() -> Self {
        // Seed the status line so it has a value before the first tick.
        let initial_status = STATUS_MESSAGES
            .first()
            .unwrap_or(&"Etat: pret.")
            .to_string();

        Self {
            name: constant::APP_NAME.to_string(),
            version: constant::APP_VERSION.to_string(),
            session: None,
            view: View::Menu(MenuState::logged_out()),
            should_quit: false,
            menu_status: initial_status,
            menu_status_index: 0,
            menu_status_updated_at: Instant::now(),
        }
    }

    pub fn logout(&mut self) {
        // Clear auth and return to guest root menu.
        self.session = None; // TODO: clear with drop
        self.view = View::Menu(MenuState::logged_out());
    }

    pub fn tick(&mut self) {
        // Called every frame; only rotate the menu status every N seconds.
        if self.menu_status_updated_at.elapsed() < STATUS_ROTATE_EVERY {
            return;
        }

        let total = STATUS_MESSAGES.len();
        if total == 0 {
            return;
        }

        // If the app was paused or lagged, advance by multiple steps.
        let steps = (self.menu_status_updated_at.elapsed().as_secs()
            / STATUS_ROTATE_EVERY.as_secs())
        .max(1) as usize;
        self.menu_status_index = (self.menu_status_index + steps) % total;
        self.menu_status = STATUS_MESSAGES[self.menu_status_index].to_string();
        self.menu_status_updated_at = Instant::now();
    }

    pub fn menu_status(&self) -> &str {
        // Small accessor so draw code doesn't touch internal fields.
        &self.menu_status
    }

    pub fn menu_entries(&self, kind: MenuPageKind) -> Vec<MenuEntry> {
        // Build the menu items for the active page.
        match kind {
            MenuPageKind::LoggedOut => vec![
                MenuEntry::signup("Sign Up"),
                MenuEntry::login("Log In"),
                MenuEntry::quit("Quit"),
            ],
            MenuPageKind::LoggedIn => vec![MenuEntry::logout("Logout")],
        }
    }

    pub fn activate_menu_selection(&mut self) {
        // Resolve the current menu selection and execute its action.
        let (kind, selected) = match &self.view {
            View::Menu(menu) => (menu.current().kind, menu.current().selected),
            _ => return,
        };

        let entries = self.menu_entries(kind);
        if entries.is_empty() {
            return;
        }

        let idx = selected.min(entries.len().saturating_sub(1));
        let action = entries[idx].action.clone();
        self.apply_menu_action(action);
    }

    pub fn apply_menu_action(&mut self, action: MenuAction) {
        // Centralized action handler so menu logic is easy to extend.
        match action {
            MenuAction::Push(kind) => {
                if let View::Menu(menu) = &mut self.view {
                    menu.push(kind);
                }
            }
            MenuAction::Signup => {
                // Open the signup form (not implemented yet, so just goes to login).
                let return_menu = match &self.view {
                    View::Menu(menu) => menu.clone(),
                    _ => MenuState::logged_out(),
                };
                self.view = View::Signup(SignupFormState::new(return_menu));
            }
            MenuAction::Login => {
                // Open the login form, with the current menu as the return target.
                let return_menu = match &self.view {
                    View::Menu(menu) => menu.clone(),
                    _ => MenuState::logged_out(),
                };
                self.view = View::Login(LoginFormState::new(return_menu));
            }
            MenuAction::Back => {
                // Back pops the menu stack; at root it just stays put.
                if let View::Menu(menu) = &mut self.view {
                    menu.pop();
                }
            }
            MenuAction::Quit => {
                self.should_quit = true;
            }
            MenuAction::Logout => {
                self.logout();
            }
        }
    }
}
