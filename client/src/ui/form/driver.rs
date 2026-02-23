use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use secrecy::SecretSlice;

use crate::core;
use crate::ui::form::view::SignupField;
use crate::ui::{
    app::App,
    form::view::{FormKind, LoginField},
    menu::view::MenuState,
    view::View,
};

pub fn handle_login_key(app: &mut App, key: KeyEvent) {
    // Login form input username then password then submit
    let mut action = LoginAction::None;

    {
        let form = match &mut app.view {
            View::Form(form) => form,
            _ => return,
        };
        let return_menu = form.return_menu.clone();
        let error = &mut form.error;
        let login = match &mut form.kind {
            FormKind::Login(login) => login,
            _ => return,
        };

        match key.code {
            KeyCode::Esc => {
                action = LoginAction::Back(return_menu);
            }
            KeyCode::Enter => {
                *error = None;
                match login.active {
                    LoginField::Username => {
                        if login.username.trim().is_empty() {
                            *error = Some("Username required.".to_string());
                        } else {
                            login.active = LoginField::Password;
                        }
                    }
                    LoginField::Password => {
                        let username = login.username.trim();
                        if username.is_empty() {
                            *error = Some("Username required.".to_string());
                            login.active = LoginField::Username;
                        } else if login.password_is_empty() {
                            *error = Some("Password required.".to_string());
                        } else {
                            action = LoginAction::Submit {
                                username: username.to_string(),
                                password: login.take_password(),
                            };
                        }
                    }
                }
            }
            KeyCode::Backspace => match login.active {
                LoginField::Username => {
                    login.username.pop();
                }
                LoginField::Password => {
                    login.pop_password_char();
                }
            },
            KeyCode::Char(ch) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    match login.active {
                        LoginField::Username => login.username.push(ch),
                        LoginField::Password => login.push_password_char(ch),
                    }
                }
            }
            _ => {}
        }
    }

    match action {
        LoginAction::None => {}
        LoginAction::Back(menu) => {
            app.view = View::Menu(menu);
        }
        LoginAction::Submit { username, password } => {
            // Submit credentials to core auth and update UI state based on result
            match core::auth::perform_login(&username, &password) {
                Ok(session) => {
                    app.session = Some(session);
                    app.view = View::Menu(MenuState::logged_in());
                }
                Err(err) => {
                    if let View::Form(form) = &mut app.view {
                        form.error = Some(err.to_string());
                        if let FormKind::Login(login) = &mut form.kind {
                            login.clear_password();
                            login.active = LoginField::Username;
                        }
                    }
                }
            }
        }
    }
}

enum LoginAction {
    None,
    Back(MenuState),
    Submit {
        username: String,
        password: SecretSlice<u8>,
    },
}

pub fn handle_signup_key(app: &mut App, key: KeyEvent) {
    // Signup form input username then password then confirm password then submit
    let mut action = SignupAction::None;

    {
        let form = match &mut app.view {
            View::Form(form) => form,
            _ => return,
        };
        let return_menu = form.return_menu.clone();
        let error = &mut form.error;
        let signup = match &mut form.kind {
            FormKind::Signup(signup) => signup,
            _ => return,
        };

        match key.code {
            KeyCode::Esc => {
                action = SignupAction::Back(return_menu);
            }
            KeyCode::Enter => {
                *error = None;
                match signup.active {
                    SignupField::Username => {
                        if signup.username.trim().is_empty() {
                            *error = Some("Username required.".to_string());
                        } else {
                            signup.active = SignupField::Password;
                        }
                    }
                    SignupField::Password => {
                        if signup.password_is_empty() {
                            *error = Some("Password required.".to_string());
                        } else {
                            signup.active = SignupField::ConfirmPassword;
                        }
                    }
                    SignupField::ConfirmPassword => {
                        if signup.confirm_is_empty() {
                            *error = Some("Confirm password required.".to_string());
                        } else if !signup.passwords_match() {
                            *error = Some("Passwords do not match.".to_string());
                            signup.clear_passwords();
                            signup.active = SignupField::Password;
                        } else {
                            action = SignupAction::Submit {
                                username: signup.username.trim().to_string(),
                                password: signup.take_password(),
                            };
                        }
                    }
                }
            }
            KeyCode::Backspace => match signup.active {
                SignupField::Username => {
                    signup.username.pop();
                }
                SignupField::Password => {
                    signup.pop_password_char();
                }
                SignupField::ConfirmPassword => {
                    signup.pop_confirm_char();
                }
            },
            KeyCode::Char(ch) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    match signup.active {
                        SignupField::Username => signup.username.push(ch),
                        SignupField::Password => signup.push_password_char(ch),
                        SignupField::ConfirmPassword => signup.push_confirm_char(ch),
                    }
                }
            }
            _ => {}
        }
    }

    match action {
        SignupAction::None => {}
        SignupAction::Back(menu) => {
            app.view = View::Menu(menu);
        }
        SignupAction::Submit { username, password } => {
            match core::auth::perform_signup(&username, &password) {
                Ok(_) => {
                    app.view = View::Menu(MenuState::logged_out());
                    // TODO : Afficher message de confirmation de création
                    // Sur le menu de retour (logged_out()) à la place du "status" en footer
                }
                Err(err) => {
                    if let View::Form(form) = &mut app.view {
                        form.error = Some(err.to_string());
                        if let FormKind::Signup(signup) = &mut form.kind {
                            signup.clear_passwords();
                            signup.active = SignupField::Username;
                        }
                    }
                }
            }
        }
    }
}

enum SignupAction {
    None,
    Back(MenuState),
    Submit {
        username: String,
        password: SecretSlice<u8>,
    },
}
