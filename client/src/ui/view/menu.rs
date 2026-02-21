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
    pub(crate) fn push(label: impl Into<String>, kind: MenuPageKind) -> Self {
        // Navigate into a submenu.
        Self {
            label: label.into(),
            action: MenuAction::Push(kind),
        }
    }

    pub(crate) fn signup(label: impl Into<String>) -> Self {
        // Open the signup form (not implemented yet, so just goes to login).
        Self {
            label: label.into(),
            action: MenuAction::Signup,
        }
    }

    pub(crate) fn login(label: impl Into<String>) -> Self {
        // Open the login form.
        Self {
            label: label.into(),
            action: MenuAction::Login,
        }
    }

    pub(crate) fn back(label: impl Into<String>) -> Self {
        // Go back up one level.
        Self {
            label: label.into(),
            action: MenuAction::Back,
        }
    }

    pub(crate) fn quit(label: impl Into<String>) -> Self {
        // Quit the app.
        Self {
            label: label.into(),
            action: MenuAction::Quit,
        }
    }

    pub(crate) fn logout(label: impl Into<String>) -> Self {
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
