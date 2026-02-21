use secrecy::{ExposeSecret, ExposeSecretMut, SecretBox, SecretSlice};

use super::menu::MenuState;

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
