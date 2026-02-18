use inquire::{InquireError, Password, Select, Text};
use openmls::group::GroupId;

use crate::ui::cli::error::UiError;

// Helpers

fn prompt_text(label: &str, err: UiError) -> Result<String, UiError> {
    Text::new(label).prompt().map_err(|_| err)
}

fn prompt_password(label: &str, confirm: bool, err: UiError) -> Result<String, UiError> {
    let mut p = Password::new(label);
    if !confirm {
        p = p.without_confirmation();
    }
    p.prompt().map_err(|_| err)
}

// Public functions
// Auth

pub fn signup() -> Result<(String, String), UiError> {
    let username = prompt_text("Enter your username:", UiError::Username)?;
    let password = prompt_password("Enter your password:", true, UiError::Password)?;
    Ok((username, password))
}

pub fn login() -> Result<(String, String), UiError> {
    let username = prompt_text("Enter your username:", UiError::Username)?;
    let password = prompt_password("Enter your password:", false, UiError::Password)?;
    Ok((username, password))
}

// Groups

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

    let selection = match Select::new("Select a group:", options).prompt() {
        Ok(selection) => selection,
        Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => return,
        Err(e) => {
            eprintln!("");
            eprintln!("Prompt error: {e}");
            return;
        }
    };

    match groups.into_iter().find(|g| g.name == selection) {
        Some(group) => show_chat(group),
        None => println!("Group not found"),
    }
}

pub fn invite_username() -> Result<String, UiError> {
    prompt_text("Enter the username to invite:", UiError::Username)
}

pub fn group_name() -> Result<String, UiError> {
    prompt_text("Enter the desired group name:", UiError::GroupName)
}

fn show_chat(group: Group) {
    println!("Showing chat for group ID: {:?}", group.id);
}
