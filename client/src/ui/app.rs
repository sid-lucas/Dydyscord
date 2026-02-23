use std::time::{Duration, Instant};

use crate::{auth::session::Session, config::constant, ui::form::view::FormState};

use super::view::{MenuAction, MenuEntry, MenuPageKind, MenuState, View};

const STATUS_ROTATE_EVERY: Duration = Duration::from_secs(1);
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
    pub view: View,
    pub should_quit: bool,
    menu_status: String,
    menu_status_index: usize,
    menu_status_updated_at: Instant,
}

impl App {
    pub fn new() -> Self {
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
        self.session = None; // TODO: clear with drop
        self.view = View::Menu(MenuState::logged_out());
    }

    pub fn tick(&mut self) {
        if self.menu_status_updated_at.elapsed() < STATUS_ROTATE_EVERY {
            return;
        }

        let total = STATUS_MESSAGES.len();
        if total == 0 {
            return;
        }

        let steps = (self.menu_status_updated_at.elapsed().as_secs()
            / STATUS_ROTATE_EVERY.as_secs())
        .max(1) as usize;
        self.menu_status_index = (self.menu_status_index + steps) % total;
        self.menu_status = STATUS_MESSAGES[self.menu_status_index].to_string();
        self.menu_status_updated_at = Instant::now();
    }

    pub fn menu_status(&self) -> &str {
        &self.menu_status
    }

    pub fn menu_entries(&self, kind: MenuPageKind) -> Vec<MenuEntry> {
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
        match action {
            MenuAction::Push(kind) => {
                if let View::Menu(menu) = &mut self.view {
                    menu.push(kind);
                }
            }
            MenuAction::Signup => {
                // Get the current menu to be able to get back to it : return_menu
                let return_menu = match &self.view {
                    View::Menu(menu) => menu.clone(),
                    _ => MenuState::logged_out(),
                };
                // Open the signup form
                self.view = View::Form(FormState::signup(return_menu));
            }
            MenuAction::Login => {
                // Get the current menu to be able to get back to it : return_menu
                let return_menu = match &self.view {
                    View::Menu(menu) => menu.clone(),
                    _ => MenuState::logged_out(),
                };
                // Open the login form
                self.view = View::Form(FormState::login(return_menu));
            }
            MenuAction::Back => {
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
