use std::time::{Duration, Instant};

use super::chat::ChatRoom;

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

#[derive(Clone, Debug)]
pub struct App {
    // Current user (used in headers and labels).
    pub user_name: String,
    // All chatrooms loaded in memory.
    pub rooms: Vec<ChatRoom>,
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
    pub fn new(user_name: String, rooms: Vec<ChatRoom>) -> Self {
        // Seed the status line so it has a value before the first tick.
        let initial_status = STATUS_MESSAGES
            .first()
            .unwrap_or(&"Etat: pret.")
            .to_string();

        Self {
            user_name,
            rooms,
            view: View::Menu(MenuState::root()),
            should_quit: false,
            menu_status: initial_status,
            menu_status_index: 0,
            menu_status_updated_at: Instant::now(),
        }
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
                MenuEntry::push("Sign up", MenuPageKind::Signup),
                MenuEntry::push("Log in", MenuPageKind::Login),
                MenuEntry::quit("Quit"),
            ],
            MenuPageKind::LoggedIn => {
                let mut entries = vec![
                    MenuEntry::push("Add a friend", MenuPageKind::Signup),
                    MenuEntry::push("Create a group", MenuPageKind::Signup),
                    MenuEntry::push("Browse groups", MenuPageKind::Signup),
                    MenuEntry::push("Fetch welcome", MenuPageKind::Signup),
                    MenuEntry::push("Test session", MenuPageKind::Signup),
                    MenuEntry::quit("Log out"),
                ];

                // Add an entry for each chatroom.
                for (index, room) in self.rooms.iter().enumerate() {
                    entries.insert(
                        0,
                        MenuEntry::chat(format!("Chat: {}", room.name), index),
                    );
                }

                entries
            }
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
            MenuAction::OpenChat(index) => {
                // Switch to chat view for the selected room.
                self.view = View::Chat { room_index: index };
            }
            MenuAction::Info { title, body } => {
                // Info panels keep a copy of the menu state so we can return.
                let return_menu = match &self.view {
                    View::Menu(menu) => menu.clone(),
                    _ => return,
                };
                self.view = View::Info(InfoState {
                    title: title.to_string(),
                    body: body.iter().map(|line| (*line).to_string()).collect(),
                    return_menu,
                });
            }
            MenuAction::Back => {
                // Back pops the menu stack or quits if we're at the root.
                if let View::Menu(menu) = &mut self.view {
                    if !menu.pop() {
                        self.should_quit = true;
                    }
                }
            }
            MenuAction::Quit => {
                self.should_quit = true;
            }
        }
    }

    pub fn back_to_chatrooms(&mut self, selected: usize) {
        // Keep the selected room highlighted when returning from a chat.
        self.view = View::Menu(MenuState::logged_out());
    }
}

#[derive(Clone, Debug)]
pub enum View {
    // Menu with nested submenus.
    Menu(MenuState),
    // Chat screen for a specific room index.
    Chat { room_index: usize },
    // Info popup that returns to the previous menu.
    Info(InfoState),
}

#[derive(Clone, Debug)]
pub struct MenuState {
    // Stack of menu pages to allow deep navigation.
    pub stack: Vec<MenuFrame>,
}

impl MenuState {
    pub fn logged_out() -> Self {
        // Start at the logged out menu.
        Self {
            stack: vec![MenuFrame {
                kind: MenuPageKind::Signup,
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
    Signup,
    Login,
}

impl MenuPageKind {
    pub fn title(self) -> &'static str {
        // Display titles for each menu page.
        match self {
            MenuPageKind::LoggedOut => "Logged Out",
            MenuPageKind::LoggedIn => "Logged In",
            MenuPageKind::Signup => "Sign Up",
            MenuPageKind::Login => "Log In",
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

    fn chat(label: impl Into<String>, index: usize) -> Self {
        // Open a chatroom.
        Self {
            label: label.into(),
            action: MenuAction::OpenChat(index),
        }
    }

    fn info(label: impl Into<String>, title: &'static str, body: &'static [&'static str]) -> Self {
        // Show an info screen.
        Self {
            label: label.into(),
            action: MenuAction::Info { title, body },
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
}

#[derive(Clone, Debug)]
pub enum MenuAction {
    // Go into a submenu.
    Push(MenuPageKind),
    // Open a chatroom by index.
    OpenChat(usize),
    // Show an info panel.
    Info {
        title: &'static str,
        body: &'static [&'static str],
    },
    // Quit the app.
    Quit,
    // Go back a level.
    Back,
}

#[derive(Clone, Debug)]
pub struct InfoState {
    // Info title shown in the header.
    pub title: String,
    // Info body lines.
    pub body: Vec<String>,
    // Snapshot of menu state for the "back" action.
    pub return_menu: MenuState,
}
