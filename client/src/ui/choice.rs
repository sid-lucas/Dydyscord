use inquire_derive::Selectable;
use std::fmt;

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
    FetchWelcome,
    TestSession,
    Logout,
}

impl fmt::Display for LoggedInChoice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoggedInChoice::AddFriend => write!(f, "Add a friend"),
            LoggedInChoice::CreateGroup => write!(f, "Create a group"),
            LoggedInChoice::FetchWelcome => write!(f, "Fetch welcome"),
            LoggedInChoice::TestSession => write!(f, "Test session"),
            LoggedInChoice::Logout => write!(f, "Log Out"),
        }
    }
}

pub fn prompt_logged_out() -> LoggedOutChoice {
    LoggedOutChoice::select("Choose an option:")
        .prompt()
        .expect("An error occurred")
}

pub fn prompt_logged_in() -> LoggedInChoice {
    LoggedInChoice::select("Choose an option:")
        .prompt()
        .expect("An error occurred")
}
