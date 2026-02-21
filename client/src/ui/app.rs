use std::time::{Duration, Instant};

use secrecy::{ExposeSecret, ExposeSecretMut, SecretBox, SecretSlice};

use crate::{auth::session::Session, config::constant};

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

#[derive(Clone, Debug)]
pub enum View {
    // Menu with nested submenus.
    Menu(MenuState),
    // Simple login form.
    Login(LoginFormState),
    Signup(SignupFormState),
}

#[derive(Clone, Debug)]
pub struct MenuState {
    // Stack of menu pages to allow deep navigation.
    pub stack: Vec<MenuFrame>,
}

impl MenuState {
    pub fn logged_out() -> Self {
        // Start at the root menu.
        Self {
            stack: vec![MenuFrame {
                kind: MenuPageKind::LoggedOut,
                selected: 0,
            }],
        }
    }

    pub fn logged_in() -> Self {
        // Root menu after a successful login.
        Self {
            stack: vec![MenuFrame {
                kind: MenuPageKind::LoggedIn,
                selected: 0,
            }],
        }
    }

    pub fn current(&self) -> &MenuFrame {
        // Read-only accessor for the active menu frame.
        self.stack.last().expect("menu stack should never be empty")
    }

    pub fn current_mut(&mut self) -> &mut MenuFrame {
        // Mutable accessor for the active menu frame.
        self.stack
            .last_mut()
            .expect("menu stack should never be empty")
    }

    pub fn push(&mut self, kind: MenuPageKind) {
        // Push a new submenu onto the stack.
        self.stack.push(MenuFrame { kind, selected: 0 });
    }

    pub fn pop(&mut self) -> bool {
        // Pop one submenu; return false if we're already at root.
        if self.stack.len() > 1 {
            self.stack.pop();
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
pub struct MenuFrame {
    // Which page is currently displayed.
    pub kind: MenuPageKind,
    // Which item is selected in that page.
    pub selected: usize,
}

impl MenuFrame {
    pub fn clamp(&mut self, len: usize) {
        // Keep selection index inside the list bounds.
        if len == 0 {
            self.selected = 0;
        } else if self.selected >= len {
            self.selected = len - 1;
        }
    }

    pub fn move_selection(&mut self, delta: isize, len: usize) {
        // Move selection up/down, clamped to list bounds.
        if len == 0 {
            self.selected = 0;
            return;
        }

        let next = self.selected as isize + delta;
        if next < 0 {
            self.selected = 0;
        } else if next as usize >= len {
            self.selected = len - 1;
        } else {
            self.selected = next as usize;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuPageKind {
    // Menu page identifiers.
    LoggedOut,
    LoggedIn,
}

impl MenuPageKind {
    pub fn title(self) -> &'static str {
        // Display titles for each menu page.
        match self {
            MenuPageKind::LoggedOut => "Welcome",
            MenuPageKind::LoggedIn => "Home",
        }
    }
}

#[derive(Clone, Debug)]
pub struct MenuEntry {
    // Text shown in the list.
    pub label: String,
    // Action executed when user presses Enter.
    pub action: MenuAction,
}

impl MenuEntry {
    fn push(label: impl Into<String>, kind: MenuPageKind) -> Self {
        // Navigate into a submenu.
        Self {
            label: label.into(),
            action: MenuAction::Push(kind),
        }
    }

    fn signup(label: impl Into<String>) -> Self {
        // Open the signup form (not implemented yet, so just goes to login).
        Self {
            label: label.into(),
            action: MenuAction::Signup,
        }
    }

    fn login(label: impl Into<String>) -> Self {
        // Open the login form.
        Self {
            label: label.into(),
            action: MenuAction::Login,
        }
    }

    fn back(label: impl Into<String>) -> Self {
        // Go back up one level.
        Self {
            label: label.into(),
            action: MenuAction::Back,
        }
    }

    fn quit(label: impl Into<String>) -> Self {
        // Quit the app.
        Self {
            label: label.into(),
            action: MenuAction::Quit,
        }
    }

    fn logout(label: impl Into<String>) -> Self {
        // Log out and return to guest menu.
        Self {
            label: label.into(),
            action: MenuAction::Logout,
        }
    }
}

#[derive(Clone, Debug)]
pub enum MenuAction {
    // Go into a submenu.
    Push(MenuPageKind),
    Signup,
    // Open the login form.
    Login,
    // Log out and return to guest menu.
    Logout,
    // Quit the app.
    Quit,
    // Go back a level.
    Back,
}

#[derive(Clone, Debug)]
pub struct LoginFormState {
    // Username input.
    pub username: String,
    // Password input (stored raw; masked in the UI).
    pub password: SecretBox<Vec<u8>>,
    // Which field is currently active.
    pub active: LoginField,
    // Error message shown under the form.
    pub error: Option<String>,
    // Snapshot of menu state for the "back" action.
    pub return_menu: MenuState,
}

impl LoginFormState {
    pub fn new(return_menu: MenuState) -> Self {
        Self {
            username: String::new(),
            password: SecretBox::new(Box::new(Vec::new())),
            active: LoginField::Username,
            error: None,
            return_menu,
        }
    }
    // Count password characters for masking
    pub fn password_len(&self) -> usize {
        let bytes = self.password.expose_secret();
        match std::str::from_utf8(bytes) {
            Ok(text) => text.chars().count(),
            Err(_) => bytes.len(),
        }
    }
    // Check whether the password buffer is empty
    pub fn password_is_empty(&self) -> bool {
        self.password.expose_secret().is_empty()
    }
    // Append a character to the password buffer
    pub fn push_password_char(&mut self, ch: char) {
        let bytes = self.password.expose_secret_mut();
        let mut buf = [0u8; 4];
        let slice = ch.encode_utf8(&mut buf);
        bytes.extend_from_slice(slice.as_bytes());
    }

    // Remove the last character from the password buffer
    pub fn pop_password_char(&mut self) {
        let bytes = self.password.expose_secret_mut();
        if bytes.is_empty() {
            return;
        }
        let mut idx = bytes.len().saturating_sub(1);
        while idx > 0 && (bytes[idx] & 0b1100_0000) == 0b1000_0000 {
            idx -= 1;
        }
        bytes.truncate(idx);
    }

    // Clear the password buffer
    pub fn clear_password(&mut self) {
        self.password.expose_secret_mut().clear();
    }

    // Move the password out as a fixed-size secret slice
    pub fn take_password(&mut self) -> SecretSlice<u8> {
        let bytes = std::mem::take(self.password.expose_secret_mut());
        SecretSlice::from(bytes)
    }
}

pub struct SignupFormState {
    // Username input.
    pub username: String,
    // Password input (stored raw; masked in the UI).
    pub password: SecretBox<Vec<u8>>,
    // Confirm password input (stored raw; masked in the UI).
    pub confirm_password: SecretBox<Vec<u8>>,
    // Which field is currently active.
    pub active: SignupField,
    // Error message shown under the form.
    pub error: Option<String>,
    // Snapshot of menu state for the "back" action.
    pub return_menu: MenuState,
}

impl SignupFormState {
    pub fn new(return_menu: MenuState) -> Self {
        Self {
            username: String::new(),
            password: SecretBox::new(Box::new(Vec::new())),
            confirm_password: SecretBox::new(Box::new(Vec::new())),
            active: SignupField::Username,
            error: None,
            return_menu,
        }
    }
    // TODO: factoriser avec confirm_password_len ET polymorphiser avec LoginFormState::password_len
    // Check whether the password buffer is empty
    pub fn password_is_empty(&self) -> bool {
        self.password.expose_secret().is_empty()
    }

    // Check whether the confirm password buffer is empty
    pub fn confirm_is_empty(&self) -> bool {
        self.confirm_password.expose_secret().is_empty()
    }

    // Count password characters for masking
    pub fn password_len(&self) -> usize {
        let bytes = self.password.expose_secret();
        match std::str::from_utf8(bytes) {
            Ok(text) => text.chars().count(),
            Err(_) => bytes.len(),
        }
    }

    // Count confirm password characters for masking
    pub fn confirm_len(&self) -> usize {
        let bytes = self.confirm_password.expose_secret();
        match std::str::from_utf8(bytes) {
            Ok(text) => text.chars().count(),
            Err(_) => bytes.len(),
        }
    }

    // Append a character to the password buffer
    pub fn push_password_char(&mut self, ch: char) {
        let bytes = self.password.expose_secret_mut();
        let mut buf = [0u8; 4];
        let slice = ch.encode_utf8(&mut buf);
        bytes.extend_from_slice(slice.as_bytes());
    }

    // Append a character to the confirm password buffer
    pub fn push_confirm_char(&mut self, ch: char) {
        let bytes = self.confirm_password.expose_secret_mut();
        let mut buf = [0u8; 4];
        let slice = ch.encode_utf8(&mut buf);
        bytes.extend_from_slice(slice.as_bytes());
    }

    // Remove the last character from the password buffer
    pub fn pop_password_char(&mut self) {
        let bytes = self.password.expose_secret_mut();
        if bytes.is_empty() {
            return;
        }
        let mut idx = bytes.len().saturating_sub(1);
        while idx > 0 && (bytes[idx] & 0b1100_0000) == 0b1000_0000 {
            idx -= 1;
        }
        bytes.truncate(idx);
    }

    // Remove the last character from the confirm password buffer
    pub fn pop_confirm_char(&mut self) {
        let bytes = self.confirm_password.expose_secret_mut();
        if bytes.is_empty() {
            return;
        }
        let mut idx = bytes.len().saturating_sub(1);
        while idx > 0 && (bytes[idx] & 0b1100_0000) == 0b1000_0000 {
            idx -= 1;
        }
        bytes.truncate(idx);
    }

    // Clear both password buffers
    pub fn clear_passwords(&mut self) {
        self.password.expose_secret_mut().clear();
        self.confirm_password.expose_secret_mut().clear();
    }

    // Check if password and confirm password match
    pub fn passwords_match(&self) -> bool {
        self.password.expose_secret() == self.confirm_password.expose_secret()
    }

    // Move the password out as a fixed-size secret slice
    pub fn take_password(&mut self) -> SecretSlice<u8> {
        let bytes = std::mem::take(self.password.expose_secret_mut());
        SecretSlice::from(bytes)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoginField {
    Username,
    Password,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignupField {
    Username,
    Password,
    ConfirmPassword,
}
