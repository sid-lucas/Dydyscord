use secrecy::{ExposeSecret, ExposeSecretMut, SecretBox, SecretSlice};

use crate::ui::menu::view::MenuState;

pub struct FormState {
    pub return_menu: MenuState,
    pub error: Option<String>,
    pub kind: FormKind,
}

pub enum FormKind {
    Login(LoginFormState),
    Signup(SignupFormState),
    GroupCreate(GroupCreateFormState),
}

impl FormState {
    // Constructor of a new form :

    pub fn login(return_menu: MenuState) -> Self {
        Self {
            return_menu,
            error: None,
            kind: FormKind::Login(LoginFormState::new()),
        }
    }

    pub fn signup(return_menu: MenuState) -> Self {
        Self {
            return_menu,
            error: None,
            kind: FormKind::Signup(SignupFormState::new()),
        }
    }

    pub fn group_create(return_menu: MenuState) -> Self {
        // TODO : PLACEHOLDER LISTE D'AMIS FIXE, CHANGER APRES IMPLEMENTATION "ADD FRIEND"
        let friends = vec![
            "alice".to_string(),
            "bob".to_string(),
            "carl".to_string(),
            "dydy".to_string(),
        ];

        Self {
            return_menu,
            error: None,
            kind: FormKind::GroupCreate(GroupCreateFormState::new(friends)),
        }
    }
}

// ========================================
// Helpers
// ========================================

fn secret_len(buf: &SecretBox<Vec<u8>>) -> usize {
    let bytes = buf.expose_secret();
    match std::str::from_utf8(bytes) {
        Ok(text) => text.chars().count(),
        Err(_) => bytes.len(),
    }
}

fn secret_is_empty(buf: &SecretBox<Vec<u8>>) -> bool {
    buf.expose_secret().is_empty()
}

fn secret_push_char(buf: &mut SecretBox<Vec<u8>>, ch: char) {
    let bytes = buf.expose_secret_mut();
    let mut tmp = [0u8; 4];
    let slice = ch.encode_utf8(&mut tmp);
    bytes.extend_from_slice(slice.as_bytes());
}

fn secret_pop_char(buf: &mut SecretBox<Vec<u8>>) {
    let bytes = buf.expose_secret_mut();
    if bytes.is_empty() {
        return;
    }
    let mut idx = bytes.len().saturating_sub(1);
    while idx > 0 && (bytes[idx] & 0b1100_0000) == 0b1000_0000 {
        idx -= 1;
    }
    bytes.truncate(idx);
}

fn secret_clear(buf: &mut SecretBox<Vec<u8>>) {
    buf.expose_secret_mut().clear();
}

fn secret_take(buf: &mut SecretBox<Vec<u8>>) -> SecretSlice<u8> {
    let bytes = std::mem::take(buf.expose_secret_mut());
    SecretSlice::from(bytes)
}

// ========================================
// Form: Log In
// ========================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoginField {
    Username,
    Password,
}

pub struct LoginFormState {
    pub username: String,
    pub password: SecretBox<Vec<u8>>,
    pub active: LoginField,
}

impl LoginFormState {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            password: SecretBox::new(Box::new(Vec::new())),
            active: LoginField::Username,
        }
    }

    pub fn password_len(&self) -> usize {
        secret_len(&self.password)
    }

    pub fn password_is_empty(&self) -> bool {
        secret_is_empty(&self.password)
    }

    pub fn push_password_char(&mut self, ch: char) {
        secret_push_char(&mut self.password, ch);
    }

    pub fn pop_password_char(&mut self) {
        secret_pop_char(&mut self.password);
    }

    pub fn clear_password(&mut self) {
        secret_clear(&mut self.password);
    }

    pub fn take_password(&mut self) -> SecretSlice<u8> {
        secret_take(&mut self.password)
    }
}

// ========================================
// Form: Sign Up
// ========================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignupField {
    Username,
    Password,
    ConfirmPassword,
}

pub struct SignupFormState {
    pub username: String,
    pub password: SecretBox<Vec<u8>>,
    pub confirm_password: SecretBox<Vec<u8>>,
    pub active: SignupField,
}

impl SignupFormState {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            password: SecretBox::new(Box::new(Vec::new())),
            confirm_password: SecretBox::new(Box::new(Vec::new())),
            active: SignupField::Username,
        }
    }

    pub fn password_is_empty(&self) -> bool {
        secret_is_empty(&self.password)
    }

    pub fn confirm_is_empty(&self) -> bool {
        secret_is_empty(&self.confirm_password)
    }

    pub fn password_len(&self) -> usize {
        secret_len(&self.password)
    }

    pub fn confirm_len(&self) -> usize {
        secret_len(&self.confirm_password)
    }

    pub fn push_password_char(&mut self, ch: char) {
        secret_push_char(&mut self.password, ch);
    }

    pub fn push_confirm_char(&mut self, ch: char) {
        secret_push_char(&mut self.confirm_password, ch);
    }

    pub fn pop_password_char(&mut self) {
        secret_pop_char(&mut self.password);
    }

    pub fn pop_confirm_char(&mut self) {
        secret_pop_char(&mut self.confirm_password);
    }

    pub fn clear_passwords(&mut self) {
        secret_clear(&mut self.password);
        secret_clear(&mut self.confirm_password);
    }

    pub fn passwords_match(&self) -> bool {
        self.password.expose_secret() == self.confirm_password.expose_secret()
    }

    pub fn take_password(&mut self) -> SecretSlice<u8> {
        secret_take(&mut self.password)
    }
}

// ========================================
// Form: Group create
// ========================================
// TODO Revoir le naming des struct/enum...

pub enum CreateGroupFormStep {
    Info,
    Members,
}

pub struct FriendEntry {
    pub username: String,
    pub selected: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupCreateField {
    Groupname,
}

pub struct GroupCreateFormState {
    pub step: CreateGroupFormStep,
    pub name: String,
    pub cursor: usize,
    pub friends: Vec<FriendEntry>,
}

impl GroupCreateFormState {
    pub fn new(friends: Vec<String>) -> Self {
        let friends = friends
            .into_iter()
            .map(|username| FriendEntry {
                username,
                selected: false,
            })
            .collect();

        Self {
            step: CreateGroupFormStep::Info,
            name: String::new(),
            cursor: 0,
            friends,
        }
    }

    pub fn selected_usernames(&self) -> Vec<String> {
        self.friends
            .iter()
            .filter(|f| f.selected)
            .map(|f| f.username.clone())
            .collect()
    }
}
