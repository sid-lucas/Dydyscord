use inquire::{Password, Text};
use openmls::group::GroupId;

use crate::ui::cli::error::UiError;

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

struct Group {
    id: GroupId,
    name: String,
}

pub fn browse_groups(groups: Vec<(GroupId, String)>) {
    let groups: Vec<Group> = groups
        .into_iter()
        .map(|(id, name)| Group { id, name })
        .collect();

    let options: Vec<String> = groups.iter().map(|g| g.name.clone()).collect();

    let selection = inquire::Select::new("Select a group:", options)
        .prompt()
        .expect("An error occurred");

    match groups.into_iter().find(|g| g.name == selection) {
        Some(group) => show_chat(group),
        None => println!("Group not found"),
    }
}

fn show_chat(group: Group) {
    println!("Showing chat for group ID: {:?}", group.id);
}
