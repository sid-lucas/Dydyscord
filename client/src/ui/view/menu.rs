#[derive(Clone, Debug)]
pub struct MenuState {
    pub stack: Vec<MenuFrame>,
}

impl MenuState {
    pub fn logged_out() -> Self {
        Self {
            stack: vec![MenuFrame {
                kind: MenuPageKind::LoggedOut,
                selected: 0,
            }],
        }
    }

    pub fn logged_in() -> Self {
        Self {
            stack: vec![MenuFrame {
                kind: MenuPageKind::LoggedIn,
                selected: 0,
            }],
        }
    }

    pub fn current(&self) -> &MenuFrame {
        self.stack.last().expect("menu stack should never be empty")
    }

    pub fn current_mut(&mut self) -> &mut MenuFrame {
        self.stack
            .last_mut()
            .expect("menu stack should never be empty")
    }

    pub fn push(&mut self, kind: MenuPageKind) {
        self.stack.push(MenuFrame { kind, selected: 0 });
    }

    pub fn pop(&mut self) -> bool {
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
    pub kind: MenuPageKind,
    pub selected: usize,
}

impl MenuFrame {
    pub fn clamp(&mut self, len: usize) {
        if len == 0 {
            self.selected = 0;
        } else if self.selected >= len {
            self.selected = len - 1;
        }
    }

    pub fn move_selection(&mut self, delta: isize, len: usize) {
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
    LoggedOut,
    LoggedIn,
}

impl MenuPageKind {
    pub fn title(self) -> &'static str {
        match self {
            MenuPageKind::LoggedOut => "Welcome",
            MenuPageKind::LoggedIn => "Home",
        }
    }
}

#[derive(Clone, Debug)]
pub struct MenuEntry {
    pub label: String,
    pub action: MenuAction,
}

impl MenuEntry {
    pub(crate) fn push(label: impl Into<String>, kind: MenuPageKind) -> Self {
        Self {
            label: label.into(),
            action: MenuAction::Push(kind),
        }
    }

    pub(crate) fn signup(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            action: MenuAction::Signup,
        }
    }

    pub(crate) fn login(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            action: MenuAction::Login,
        }
    }

    pub(crate) fn back(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            action: MenuAction::Back,
        }
    }

    pub(crate) fn quit(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            action: MenuAction::Quit,
        }
    }

    pub(crate) fn logout(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            action: MenuAction::Logout,
        }
    }
}

#[derive(Clone, Debug)]
pub enum MenuAction {
    Push(MenuPageKind),
    Signup,
    Login,
    Logout,
    Quit,
    Back,
}
