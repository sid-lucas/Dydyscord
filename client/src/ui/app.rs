use std::time::{Duration, Instant};

use secrecy::{ExposeSecret, ExposeSecretMut, SecretBox, SecretSlice};

use super::chat::ChatRoom;
use crate::auth::session::Session;
use crate::config::constant;
use crate::ui::cli::prompt::Group;

// How often the menu status line rotates
const STATUS_ROTATE_EVERY: Duration = Duration::from_secs(2);
// Rotating messages shown only in menu views
const STATUS_MESSAGES: &[&str] = &[
    "Etat: synchronisation des salons...",
    "Tip: utilisez PgUp/PgDn pour naviguer plus vite.",
    "Actu: nouveaux salons bientot disponibles.",
    "Info: vos messages sont sauvegardes localement (placeholder).",
    "Astuce: Esc remonte d'un niveau de menu.",
];
// How long action messages stay on screen
const ACTION_MSG_TTL: Duration = Duration::from_secs(4);

// Authentication state for the UI layer
pub struct AppAuth {
    // Session of the logged in user
    pub session: Option<Session>,
    // Simple auth flag to gate behaviors
    pub authenticated: bool,
}

impl AppAuth {
    // Build a default unauthenticated state
    pub fn new() -> Self {
        Self {
            session: None,
            authenticated: false,
        }
    }
}

// App data loaded in memory
pub struct AppData {
    // All chatrooms loaded in memory
    pub rooms: Option<Vec<ChatRoom>>,
    // Last selected group from the browse flow
    pub selected_group: Option<Group>,
}

impl AppData {
    // Build an empty data container
    pub fn new() -> Self {
        Self {
            rooms: None,
            selected_group: None,
        }
    }
}

// UI state for menus and navigation
pub struct AppUi {
    // Which screen is currently active
    pub view: View,
    // Cached status message so it stays stable across menu navigation
    menu_status: String,
    // Index into STATUS_MESSAGES
    menu_status_index: usize,
    // Last time the status was updated
    menu_status_updated_at: Instant,
    // Action message shown after user flows
    action_msg: Option<String>,
    // When the action message was set
    action_msg_set_at: Option<Instant>,
}

impl AppUi {
    // Build the initial UI state
    pub fn new() -> Self {
        let initial_status = STATUS_MESSAGES
            .first()
            .unwrap_or(&"Etat: pret.")
            .to_string();

        Self {
            view: View::Menu(MenuState::root()),
            menu_status: initial_status,
            menu_status_index: 0,
            menu_status_updated_at: Instant::now(),
            action_msg: None,
            action_msg_set_at: None,
        }
    }

    // Read-only accessor for the status line
    pub fn menu_status(&self) -> &str {
        match self.action_msg.as_deref() {
            Some(msg) => msg,
            None => &self.menu_status,
        }
    }
}

// Runtime flags that control the main loop
pub struct AppRuntime {
    // Signal to stop the app loop
    pub should_quit: bool,
    // Pending blocking action triggered by a menu selection
    pub pending_action: Option<BlockingAction>,
}

impl AppRuntime {
    // Build a runtime state with no quit signal
    pub fn new() -> Self {
        Self {
            should_quit: false,
            pending_action: None,
        }
    }
}

// Actions that require leaving ratatui to run blocking prompts
#[derive(Clone, Copy, Debug)]
pub enum BlockingAction {
    Signup,
    Login,
    AddFriend,
    CreateGroup,
    BrowseGroups,
    FetchWelcome,
    TestSession,
}

// Root application state container
pub struct App {
    // App name for display
    pub name: String,
    // App version for display
    pub version: String,
    // Authentication-related state
    pub auth: AppAuth,
    // Loaded data for menus and chat
    pub data: AppData,
    // UI state for navigation and status
    pub ui: AppUi,
    // Runtime flags for the loop
    pub runtime: AppRuntime,
}

impl App {
    // Build a fresh application state
    pub fn new() -> Self {
        Self {
            name: constant::APP_NAME.to_string(),
            version: constant::APP_VERSION.to_string(),
            auth: AppAuth::new(),
            data: AppData::new(),
            ui: AppUi::new(),
            runtime: AppRuntime::new(),
        }
    }

    // Clear authentication and return to the guest menu
    pub fn logout(&mut self) {
        self.auth.session = None;
        self.auth.authenticated = false;
        self.data.rooms = None;
        self.data.selected_group = None;
        self.ui.action_msg = None;
        self.ui.action_msg_set_at = None;
        self.ui.view = View::Menu(MenuState::root());
    }

    // Advance time-based UI state
    pub fn tick(&mut self) {
        if let Some(set_at) = self.ui.action_msg_set_at {
            if set_at.elapsed() < ACTION_MSG_TTL {
                return;
            }
            self.ui.action_msg = None;
            self.ui.action_msg_set_at = None;
        }

        // Rotate the status line only every N seconds
        if self.ui.menu_status_updated_at.elapsed() < STATUS_ROTATE_EVERY {
            return;
        }

        let total = STATUS_MESSAGES.len();
        if total == 0 {
            return;
        }

        // If the app was paused or lagged, advance by multiple steps
        let steps = (self.ui.menu_status_updated_at.elapsed().as_secs()
            / STATUS_ROTATE_EVERY.as_secs())
        .max(1) as usize;
        self.ui.menu_status_index = (self.ui.menu_status_index + steps) % total;
        self.ui.menu_status = STATUS_MESSAGES[self.ui.menu_status_index].to_string();
        self.ui.menu_status_updated_at = Instant::now();
    }

    // Read-only accessor for status line text
    pub fn menu_status(&self) -> &str {
        self.ui.menu_status()
    }

    // Store a transient action message for the status line
    pub fn set_action_msg(&mut self, msg: impl Into<String>) {
        self.ui.action_msg = Some(msg.into());
        self.ui.action_msg_set_at = Some(Instant::now());
    }

    // Get the current session if it exists
    pub fn session(&self) -> Option<&Session> {
        self.auth.session.as_ref()
    }

    // Update the session and authenticated flag together
    pub fn set_session(&mut self, session: Option<Session>) {
        self.auth.session = session;
        self.auth.authenticated = self.auth.session.is_some();
    }

    // Get the last selected group from the browse flow
    pub fn selected_group(&self) -> Option<&Group> {
        self.data.selected_group.as_ref()
    }

    // Store the selected group from the browse flow
    pub fn set_selected_group(&mut self, group: Group) {
        self.data.selected_group = Some(group);
    }

    // Build menu entries for the active page
    pub fn menu_entries(&self, kind: MenuPageKind) -> Vec<MenuEntry> {
        match kind {
            MenuPageKind::RootGuest => vec![
                // Guest landing page actions
                MenuEntry::signup("Sign Up"),
                MenuEntry::login("Log In"),
                MenuEntry::quit("Quit"),
            ],
            MenuPageKind::RootAuthed => vec![
                // Authenticated landing page actions
                MenuEntry::add_friend("Add Friend"),
                MenuEntry::create_group("Create Group"),
                MenuEntry::browse_groups("Browse Groups"),
                MenuEntry::fetch_welcome("Fetch Welcome"),
                MenuEntry::test_session("Test Session"),
                MenuEntry::logout("Logout"),
            ],
            MenuPageKind::Chatrooms => {
                // Chatrooms are fully dynamic; this list is built from app data
                let mut entries = Vec::new();

                match self.data.rooms.as_ref() {
                    Some(rooms) if !rooms.is_empty() => {
                        for (idx, room) in rooms.iter().enumerate() {
                            entries.push(MenuEntry::chat(format!("#{}", room.name), idx));
                        }
                    }
                    _ => {
                        entries.push(MenuEntry::info(
                            "Aucun salon (placeholder)",
                            "Aucun salon",
                            &[
                                "Cette section est prete pour des salons dynamiques.",
                                "Ajoute une source de donnees pour hydrater la liste.",
                            ],
                        ));
                    }
                }

                // Always add a way back at the end
                entries.push(MenuEntry::back("Back"));
                entries
            }
            MenuPageKind::Settings => vec![
                // Placeholder entries show where future settings will go
                MenuEntry::info(
                    "Account (placeholder)",
                    "Account",
                    &[
                        "Gestion du profil, identite et securite.",
                        "Placeholder pour branchements futurs.",
                    ],
                ),
                MenuEntry::push("Appearance", MenuPageKind::Appearance),
                MenuEntry::info(
                    "Notifications (placeholder)",
                    "Notifications",
                    &[
                        "Parametrage des alertes et des sons.",
                        "Placeholder pour branchements futurs.",
                    ],
                ),
                MenuEntry::back("Back"),
            ],
            MenuPageKind::Appearance => vec![
                // Appearance is a deeper submenu to show menu depth
                MenuEntry::info(
                    "Theme: Tomato (placeholder)",
                    "Theme",
                    &[
                        "Choix de theme (light/dark/custom).",
                        "Placeholder pour branchements futurs.",
                    ],
                ),
                MenuEntry::info(
                    "Density: Comfortable (placeholder)",
                    "Density",
                    &[
                        "Reglage de densite et spacing.",
                        "Placeholder pour branchements futurs.",
                    ],
                ),
                MenuEntry::back("Back"),
            ],
            MenuPageKind::Tools => vec![
                // Tools is another submenu to show depth
                MenuEntry::push("Diagnostics", MenuPageKind::Diagnostics),
                MenuEntry::info(
                    "Import (placeholder)",
                    "Import",
                    &[
                        "Import d'historique ou de donnees.",
                        "Placeholder pour branchements futurs.",
                    ],
                ),
                MenuEntry::back("Back"),
            ],
            MenuPageKind::Diagnostics => vec![
                // Diagnostics shows extra depth with placeholder actions
                MenuEntry::info(
                    "Ping server (placeholder)",
                    "Ping",
                    &[
                        "Verification de connectivite.",
                        "Placeholder pour branchements futurs.",
                    ],
                ),
                MenuEntry::info(
                    "Logs (placeholder)",
                    "Logs",
                    &[
                        "Affichage des logs et traces.",
                        "Placeholder pour branchements futurs.",
                    ],
                ),
                MenuEntry::back("Back"),
            ],
            MenuPageKind::About => vec![
                // About is just info entries for now
                MenuEntry::info(
                    "Roadmap (placeholder)",
                    "Roadmap",
                    &[
                        "Fonctionnalites a venir.",
                        "Placeholder pour branchements futurs.",
                    ],
                ),
                MenuEntry::info(
                    "Credits (placeholder)",
                    "Credits",
                    &[
                        "Equipe, contributeurs, licences.",
                        "Placeholder pour branchements futurs.",
                    ],
                ),
                MenuEntry::back("Back"),
            ],
        }
    }

    pub fn activate_menu_selection(&mut self) {
        // Resolve the current menu selection and execute its action
        let (kind, selected) = match &self.ui.view {
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
        // Centralized action handler so menu logic is easy to extend
        match action {
            MenuAction::Push(kind) => {
                if let View::Menu(menu) = &mut self.ui.view {
                    menu.push(kind);
                }
            }
            MenuAction::Signup => {
                self.runtime.pending_action = Some(BlockingAction::Signup);
            }
            MenuAction::Login => {
                // Open the login form, with the current menu as the return target
                let return_menu = match &self.ui.view {
                    View::Menu(menu) => menu.clone(),
                    _ => MenuState::root(),
                };
                self.ui.view = View::Login(LoginFormState::new(return_menu));
            }
            MenuAction::AddFriend => {
                self.runtime.pending_action = Some(BlockingAction::AddFriend);
            }
            MenuAction::CreateGroup => {
                self.runtime.pending_action = Some(BlockingAction::CreateGroup);
            }
            MenuAction::BrowseGroups => {
                self.runtime.pending_action = Some(BlockingAction::BrowseGroups);
            }
            MenuAction::FetchWelcome => {
                self.runtime.pending_action = Some(BlockingAction::FetchWelcome);
            }
            MenuAction::TestSession => {
                self.runtime.pending_action = Some(BlockingAction::TestSession);
            }
            MenuAction::OpenChat(index) => {
                // Switch to chat view for the selected room
                self.ui.view = View::Chat { room_index: index };
            }
            MenuAction::Info { title, body } => {
                // Info panels keep a copy of the menu state so we can return
                let return_menu = match &self.ui.view {
                    View::Menu(menu) => menu.clone(),
                    _ => return,
                };
                self.ui.view = View::Info(InfoState {
                    title: title.to_string(),
                    body: body.iter().map(|line| (*line).to_string()).collect(),
                    return_menu,
                });
            }
            MenuAction::Back => {
                // Back pops the menu stack; at root it just stays put
                if let View::Menu(menu) = &mut self.ui.view {
                    menu.pop();
                }
            }
            MenuAction::Quit => {
                self.runtime.should_quit = true;
            }
            MenuAction::Logout => {
                self.logout();
            }
        }
    }

    pub fn back_to_chatrooms(&mut self, selected: usize) {
        // Keep the selected room highlighted when returning from a chat
        self.ui.view = View::Menu(MenuState::chatrooms(selected));
    }
}

// High-level view variants for the UI
#[derive(Clone, Debug)]
pub enum View {
    // Menu with nested submenus
    Menu(MenuState),
    // Chat screen for a specific room index
    Chat { room_index: usize },
    // Info popup that returns to the previous menu
    Info(InfoState),
    // Simple login form
    Login(LoginFormState),
}

// Stack-driven navigation state for menu pages
#[derive(Clone, Debug)]
pub struct MenuState {
    // Stack of menu pages to allow deep navigation
    pub stack: Vec<MenuFrame>,
}

// Menu stack helpers for pushing and popping pages
impl MenuState {
    pub fn root() -> Self {
        // Start at the root menu
        Self {
            stack: vec![MenuFrame {
                kind: MenuPageKind::RootGuest,
                selected: 0,
            }],
        }
    }

    pub fn authed_root() -> Self {
        // Root menu after a successful login
        Self {
            stack: vec![MenuFrame {
                kind: MenuPageKind::RootAuthed,
                selected: 0,
            }],
        }
    }

    pub fn chatrooms(selected: usize) -> Self {
        // Jump directly into the Chatrooms submenu (used when leaving a chat)
        Self {
            stack: vec![
                MenuFrame {
                    kind: MenuPageKind::RootAuthed,
                    selected: 0,
                },
                MenuFrame {
                    kind: MenuPageKind::Chatrooms,
                    selected,
                },
            ],
        }
    }

    pub fn current(&self) -> &MenuFrame {
        // Read-only accessor for the active menu frame
        self.stack.last().expect("menu stack should never be empty")
    }

    pub fn current_mut(&mut self) -> &mut MenuFrame {
        // Mutable accessor for the active menu frame
        self.stack
            .last_mut()
            .expect("menu stack should never be empty")
    }

    pub fn push(&mut self, kind: MenuPageKind) {
        // Push a new submenu onto the stack
        self.stack.push(MenuFrame { kind, selected: 0 });
    }

    pub fn pop(&mut self) -> bool {
        // Pop one submenu; return false if we're already at root
        if self.stack.len() > 1 {
            self.stack.pop();
            true
        } else {
            false
        }
    }
}

// A single menu frame with selection state
#[derive(Clone, Debug)]
pub struct MenuFrame {
    // Which page is currently displayed
    pub kind: MenuPageKind,
    // Which item is selected in that page
    pub selected: usize,
}

// Helpers for keeping menu selection valid
impl MenuFrame {
    pub fn clamp(&mut self, len: usize) {
        // Keep selection index inside the list bounds
        if len == 0 {
            self.selected = 0;
        } else if self.selected >= len {
            self.selected = len - 1;
        }
    }

    pub fn move_selection(&mut self, delta: isize, len: usize) {
        // Move selection up/down, clamped to list bounds
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

// Identifiers for each menu page
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuPageKind {
    // Menu page identifiers
    RootGuest,
    RootAuthed,
    Chatrooms,
    Settings,
    Appearance,
    Tools,
    Diagnostics,
    About,
}

// Human-readable titles for each menu page
impl MenuPageKind {
    pub fn title(self) -> &'static str {
        // Display titles for each menu page
        match self {
            MenuPageKind::RootGuest => "Welcome",
            MenuPageKind::RootAuthed => "Home",
            MenuPageKind::Chatrooms => "Chatrooms",
            MenuPageKind::Settings => "Settings",
            MenuPageKind::Appearance => "Appearance",
            MenuPageKind::Tools => "Tools",
            MenuPageKind::Diagnostics => "Diagnostics",
            MenuPageKind::About => "About",
        }
    }
}

// A rendered menu item with label and action
#[derive(Clone, Debug)]
pub struct MenuEntry {
    // Text shown in the list
    pub label: String,
    // Action executed when user presses Enter
    pub action: MenuAction,
}

// Constructors for common menu entry patterns
impl MenuEntry {
    fn push(label: impl Into<String>, kind: MenuPageKind) -> Self {
        // Navigate into a submenu
        Self {
            label: label.into(),
            action: MenuAction::Push(kind),
        }
    }

    fn chat(label: impl Into<String>, index: usize) -> Self {
        // Open a chatroom
        Self {
            label: label.into(),
            action: MenuAction::OpenChat(index),
        }
    }

    fn info(label: impl Into<String>, title: &'static str, body: &'static [&'static str]) -> Self {
        // Show an info screen
        Self {
            label: label.into(),
            action: MenuAction::Info { title, body },
        }
    }

    fn signup(label: impl Into<String>) -> Self {
        // Open the signup flow
        Self {
            label: label.into(),
            action: MenuAction::Signup,
        }
    }

    fn login(label: impl Into<String>) -> Self {
        // Open the login form
        Self {
            label: label.into(),
            action: MenuAction::Login,
        }
    }

    fn add_friend(label: impl Into<String>) -> Self {
        // Open the add friend flow
        Self {
            label: label.into(),
            action: MenuAction::AddFriend,
        }
    }

    fn create_group(label: impl Into<String>) -> Self {
        // Open the create group flow
        Self {
            label: label.into(),
            action: MenuAction::CreateGroup,
        }
    }

    fn browse_groups(label: impl Into<String>) -> Self {
        // Open the browse groups flow
        Self {
            label: label.into(),
            action: MenuAction::BrowseGroups,
        }
    }

    fn fetch_welcome(label: impl Into<String>) -> Self {
        // Open the fetch welcome flow
        Self {
            label: label.into(),
            action: MenuAction::FetchWelcome,
        }
    }

    fn test_session(label: impl Into<String>) -> Self {
        // Open the test session flow
        Self {
            label: label.into(),
            action: MenuAction::TestSession,
        }
    }

    fn back(label: impl Into<String>) -> Self {
        // Go back up one level
        Self {
            label: label.into(),
            action: MenuAction::Back,
        }
    }

    fn quit(label: impl Into<String>) -> Self {
        // Quit the app
        Self {
            label: label.into(),
            action: MenuAction::Quit,
        }
    }

    fn logout(label: impl Into<String>) -> Self {
        // Log out and return to guest menu
        Self {
            label: label.into(),
            action: MenuAction::Logout,
        }
    }
}

// Actions triggered by menu selections
#[derive(Clone, Debug)]
pub enum MenuAction {
    // Go into a submenu
    Push(MenuPageKind),
    // Open the signup flow
    Signup,
    // Open the login form
    Login,
    // Open the add friend flow
    AddFriend,
    // Open the create group flow
    CreateGroup,
    // Open the browse groups flow
    BrowseGroups,
    // Fetch welcome messages
    FetchWelcome,
    // Test the current session
    TestSession,
    // Open a chatroom by index
    OpenChat(usize),
    // Show an info panel
    Info {
        title: &'static str,
        body: &'static [&'static str],
    },
    // Quit the app
    Quit,
    // Log out and return to guest menu
    Logout,
    // Go back a level
    Back,
}

// State for a simple info page
#[derive(Clone, Debug)]
pub struct InfoState {
    // Info title shown in the header
    pub title: String,
    // Info body lines
    pub body: Vec<String>,
    // Snapshot of menu state for the "back" action
    pub return_menu: MenuState,
}

// State for the login form
#[derive(Debug)]
pub struct LoginFormState {
    // Username input
    pub username: String,
    // Password input stored as secret bytes (masked in the UI)
    pub password: SecretBox<Vec<u8>>,
    // Which field is currently active
    pub active: LoginField,
    // Error message shown under the form
    pub error: Option<String>,
    // Snapshot of menu state for the "back" action
    pub return_menu: MenuState,
}

// Helpers for login form input management
impl LoginFormState {
    // Create a new empty login form state
    pub fn new(return_menu: MenuState) -> Self {
        Self {
            username: String::new(),
            password: SecretBox::new(Box::new(Vec::new())),
            active: LoginField::Username,
            error: None,
            return_menu,
        }
    }

    // Check whether the password buffer is empty
    pub fn password_is_empty(&self) -> bool {
        self.password.expose_secret().is_empty()
    }

    // Count password characters for masking
    pub fn password_len(&self) -> usize {
        let bytes = self.password.expose_secret();
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

// Manual clone to keep the password buffer secret-aware
impl Clone for LoginFormState {
    fn clone(&self) -> Self {
        Self {
            username: self.username.clone(),
            password: SecretBox::new(Box::new(self.password.expose_secret().clone())),
            active: self.active,
            error: self.error.clone(),
            return_menu: self.return_menu.clone(),
        }
    }
}

// Identify which login field is currently active
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoginField {
    Username,
    Password,
}
