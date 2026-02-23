use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use secrecy::SecretSlice;

use crate::core;
use crate::ui::form::view::{GroupCreateField, SignupField};
use crate::ui::{
    app::App,
    form::view::{FormKind, LoginField},
    menu::view::MenuState,
    view::View,
};

// ========================================
// Form: Log In
// ========================================

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
        let state = match &mut form.kind {
            FormKind::Login(state) => state,
            _ => return,
        };

        match key.code {
            KeyCode::Esc => {
                action = LoginAction::Back(return_menu);
            }
            KeyCode::Enter => {
                *error = None;
                match state.active {
                    LoginField::Username => {
                        if state.username.trim().is_empty() {
                            *error = Some("Username required.".to_string());
                        } else {
                            state.active = LoginField::Password;
                        }
                    }
                    LoginField::Password => {
                        let username = state.username.trim();
                        if username.is_empty() {
                            *error = Some("Username required.".to_string());
                            state.active = LoginField::Username;
                        } else if state.password_is_empty() {
                            *error = Some("Password required.".to_string());
                        } else {
                            action = LoginAction::Submit {
                                username: username.to_string(),
                                password: state.take_password(),
                            };
                        }
                    }
                }
            }
            KeyCode::Backspace => match state.active {
                LoginField::Username => {
                    state.username.pop();
                }
                LoginField::Password => {
                    state.pop_password_char();
                }
            },
            KeyCode::Char(ch) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    match state.active {
                        LoginField::Username => state.username.push(ch),
                        LoginField::Password => state.push_password_char(ch),
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
                        if let FormKind::Login(state) = &mut form.kind {
                            state.clear_password();
                            state.active = LoginField::Username;
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

// ========================================
// Form: Sign up
// ========================================

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
        let state = match &mut form.kind {
            FormKind::Signup(state) => state,
            _ => return,
        };

        match key.code {
            KeyCode::Esc => {
                action = SignupAction::Back(return_menu);
            }
            KeyCode::Enter => {
                *error = None;
                match state.active {
                    SignupField::Username => {
                        if state.username.trim().is_empty() {
                            *error = Some("Username required.".to_string());
                        } else {
                            state.active = SignupField::Password;
                        }
                    }
                    SignupField::Password => {
                        if state.password_is_empty() {
                            *error = Some("Password required.".to_string());
                        } else {
                            state.active = SignupField::ConfirmPassword;
                        }
                    }
                    SignupField::ConfirmPassword => {
                        if state.confirm_is_empty() {
                            *error = Some("Confirm password required.".to_string());
                        } else if !state.passwords_match() {
                            *error = Some("Passwords do not match.".to_string());
                            state.clear_passwords();
                            state.active = SignupField::Password;
                        } else {
                            action = SignupAction::Submit {
                                username: state.username.trim().to_string(),
                                password: state.take_password(),
                            };
                        }
                    }
                }
            }
            KeyCode::Backspace => match state.active {
                SignupField::Username => {
                    state.username.pop();
                }
                SignupField::Password => {
                    state.pop_password_char();
                }
                SignupField::ConfirmPassword => {
                    state.pop_confirm_char();
                }
            },
            KeyCode::Char(ch) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    match state.active {
                        SignupField::Username => state.username.push(ch),
                        SignupField::Password => state.push_password_char(ch),
                        SignupField::ConfirmPassword => state.push_confirm_char(ch),
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
                        if let FormKind::Signup(state) = &mut form.kind {
                            state.clear_passwords();
                            state.active = SignupField::Username;
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

// ========================================
// Form: Group create
// ========================================

pub fn handle_groupcreate_key(app: &mut App, key: KeyEvent) {
    // Signup form input username then password then confirm password then submit
    let mut action = GroupCreateAction::None;

    {
        let form = match &mut app.view {
            View::Form(form) => form,
            _ => return,
        };
        let return_menu = form.return_menu.clone();
        let error = &mut form.error;
        let state = match &mut form.kind {
            FormKind::GroupCreate(state) => state,
            _ => return,
        };

        match key.code {
            KeyCode::Esc => {
                action = GroupCreateAction::Back(return_menu);
            }
            KeyCode::Enter => {
                *error = None;
                match state.active {
                    GroupCreateField::Groupname => {
                        if state.groupname.trim().is_empty() {
                            *error = Some("Group name required.".to_string());
                        } else {
                            // TODO
                            //state.active = GroupCreateField::Password;
                        }
                    }
                }
            }
            KeyCode::Backspace => match state.active {
                GroupCreateField::Groupname => {
                    state.groupname.pop();
                }
            },
            KeyCode::Char(ch) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    match state.active {
                        GroupCreateField::Groupname => state.groupname.push(ch),
                    }
                }
            }
            _ => {}
        }
    }

    match action {
        GroupCreateAction::None => {}
        GroupCreateAction::Back(menu) => {
            app.view = View::Menu(menu);
        }
        GroupCreateAction::Submit { groupname } => {

            // TODO logique métier
            /*
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
             */
        }
    }
}

enum GroupCreateAction {
    None,
    Back(MenuState),
    Submit { groupname: String },
}
