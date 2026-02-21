use std::fmt;

use inquire::InquireError;
use inquire_derive::Selectable;

#[derive(Debug, Copy, Clone, Selectable)]
pub enum LoggedOutChoice {
    Signup,
    Login,
    Quit,
}

impl fmt::Display for LoggedOutChoice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoggedOutChoice::Signup => write!(f, "Sign Up"),
            LoggedOutChoice::Login => write!(f, "Log In"),
            LoggedOutChoice::Quit => write!(f, "Quit"),
        }
    }
}

#[derive(Debug, Copy, Clone, Selectable)]
pub enum LoggedInChoice {
    AddFriend,
    CreateGroup,
    BrowseGroups,
    FetchWelcome,
    TestSession,
    Logout,
}

impl fmt::Display for LoggedInChoice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoggedInChoice::AddFriend => write!(f, "Add a friend"),
            LoggedInChoice::CreateGroup => write!(f, "Create a group"),
            LoggedInChoice::BrowseGroups => write!(f, "Browse groups"),
            LoggedInChoice::FetchWelcome => write!(f, "Fetch welcome"),
            LoggedInChoice::TestSession => write!(f, "Test session"),
            LoggedInChoice::Logout => write!(f, "Log Out"),
        }
    }
}

// TODO : factoriser en prompt_choice()
pub fn prompt_logged_out() -> LoggedOutChoice {
    match LoggedOutChoice::select("Choose an option:").prompt() {
        Ok(choice) => choice,
        Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => {
            println!("");
            println!("Bye.");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("");
            eprintln!("Prompt error: {e}");
            std::process::exit(1);
        }
    }
}

pub fn prompt_logged_in() -> Option<LoggedInChoice> {
    match LoggedInChoice::select("Choose an option:").prompt() {
        Ok(choice) => Some(choice),
        Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => None,
        Err(e) => {
            eprintln!("");
            eprintln!("Prompt error: {e}");
            None
        }
    }
}
