use inquire::{Password, Text};

use crate::ui::error::UiError;

// TODO : Factoriser d'une manière ou d'une autre... répétitions....

pub fn signup() -> Result<(String, String), UiError> {
    let username = Text::new("Enter your username:")
        .prompt()
        .map_err(|_| UiError::Username)?;

    let password = Password::new("Enter your password:")
        .prompt()
        .map_err(|_| UiError::Password)?;
    Ok((username, password))
}

pub fn login() -> Result<(String, String), UiError> {
    let username = Text::new("Enter your username:")
        .prompt()
        .map_err(|_| UiError::Username)?;

    let password = Password::new("Enter your password:")
        .without_confirmation()
        .prompt()
        .map_err(|_| UiError::Password)?;

    Ok((username, password))
}

pub fn invite_username() -> Result<String, UiError> {
    Text::new("Enter the username to invite:")
        .prompt()
        .map_err(|_| UiError::Username)
}

pub fn group_name() -> Result<String, UiError> {
    Text::new("Enter the desired group name:")
        .prompt()
        .map_err(|_| UiError::GroupName)
}
