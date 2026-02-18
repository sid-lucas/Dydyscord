use crate::{
    auth, mls, storage, transport,
    ui::cli::{choice, prompt},
};

use crossterm::{
    ExecutableCommand,
    cursor::MoveTo,
    terminal::{Clear, ClearType},
};
use std::io::{Write, stdout};

use crate::app::state::AppState;
use crate::auth::session::Session;

#[derive(Clone, Copy, Debug)]
pub enum Screen {
    MenuLoggedOut,
    FlowSignup,
    FlowLogin,
    MenuLoggedIn,
    FlowCreateGroup,
    MenuBrowseGroups,
}

impl Screen {
    fn handle(self, state: &mut AppState) -> Action {
        match self {
            Screen::MenuLoggedOut => logged_out_menu(),
            Screen::FlowSignup => signup(state),
            Screen::FlowLogin => login(state),
            Screen::MenuLoggedIn => logged_in_menu(state),
            Screen::FlowCreateGroup => create_group(state),
            Screen::MenuBrowseGroups => browse_groups(state),
        }
    }
}

// Action returned by a screen to move in the menu
#[derive(Clone, Copy, Debug)]
pub enum Action {
    Stay,
    Back,
    Quit,
    Push(Screen),
    Replace(Screen),
}

// Clear terminal when navigate to another screen
fn clear_terminal() {
    let mut out = stdout();
    let _ = out.execute(Clear(ClearType::All));
    let _ = out.execute(MoveTo(0, 0));
    let _ = out.flush();
}

// Main router, all navigation is decided here
pub fn run(state: &mut AppState) {
    // Screen stack, top is current screen
    // Allow to easily return to the previous screen
    let mut stack = vec![Screen::MenuLoggedOut];

    loop {
        // Get current screen from stack top
        let screen = match stack.last().copied() {
            Some(s) => s,
            None => break,
        };

        // Display the screen on terminal
        // And retrieve the user action
        let action = screen.handle(state);

        let mut new_screen = false;

        // Apply the user action
        // Change the screen stack depending of what the user choosed
        match action {
            Action::Back => {
                stack.pop();
                new_screen = true;
                if stack.is_empty() {
                    break;
                }
            }
            Action::Quit => break,
            Action::Stay => {}
            Action::Push(s) => {
                stack.push(s);
                new_screen = true;
            }
            Action::Replace(s) => {
                stack.pop();
                stack.push(s);
                new_screen = true;
            }
        }

        if new_screen {
            clear_terminal();
            state.show_flash();
        }
    }
}

// Main menu when logged out (signup/login/quit)
fn logged_out_menu() -> Action {
    // Display the choice, and wait for user action
    match choice::prompt_logged_out() {
        choice::LoggedOutChoice::Signup => Action::Push(Screen::FlowSignup),
        choice::LoggedOutChoice::Login => Action::Push(Screen::FlowLogin),
        choice::LoggedOutChoice::Quit => Action::Quit,
    }
}

// Main menu when logged in, each choice goes to a screen or runs once
fn logged_in_menu(state: &mut AppState) -> Action {
    match choice::prompt_logged_in() {
        Some(choice::LoggedInChoice::AddFriend) => {
            add_friend();
            Action::Stay
        }
        Some(choice::LoggedInChoice::CreateGroup) => Action::Push(Screen::FlowCreateGroup),
        Some(choice::LoggedInChoice::BrowseGroups) => Action::Push(Screen::MenuBrowseGroups),
        Some(choice::LoggedInChoice::FetchWelcome) => {
            fetch_welcome(state);
            Action::Stay
        }
        Some(choice::LoggedInChoice::TestSession) => {
            test_session();
            Action::Stay
        }
        Some(choice::LoggedInChoice::Logout) => {
            state.session = None;
            state.set_flash("Logged out.");
            Action::Back
        }
        None => Action::Back,
    }
}

// Signup screen, then go back
fn signup(state: &mut AppState) -> Action {
    // Ask the user for his username and password
    let (username, password) = match prompt::signup() {
        Ok((username, password)) => (username, password),
        Err(e) => {
            state.set_flash(format!("Signup failed: {e}"));
            return Action::Back;
        }
    };

    // Try to register with OPAQUE
    match auth::opaque::register(&username, &password) {
        Ok(_) => state.set_flash("Registration successful!"),
        Err(e) => state.set_flash(format!("Registration failed: {e}")),
    }

    // Go back to main menu
    Action::Back
}

// Login screen, on success go to MenuLoggedIn
fn login(state: &mut AppState) -> Action {
    // OPAQUE login with server to get intermediate token (auth token)
    let (username, password) = match prompt::login() {
        Ok((username, password)) => (username, password),
        Err(e) => {
            state.set_flash(format!("Login failed: {e}"));
            return Action::Back;
        }
    };

    // Retrieve login results and create new session with it if successful
    let mut session = match auth::opaque::login(&username, &password) {
        Ok(login_result) => Session::new(login_result),
        Err(e) => {
            state.set_flash(format!("Login failed: {e}"));
            return Action::Back;
        }
    };

    // Init local storage and get final token (session token)
    let (device_id, db_key, is_new_device) =
        match storage::database::init_device_storage(session.user_id(), session.export_key()) {
            Ok(result) => result,
            Err(e) => {
                state.set_flash(format!("Device storage initialization failed: {e}"));
                return Action::Back;
            }
        };

    // Save device and storage info in the session
    session.set_device_id(&device_id);
    session.set_db_key(db_key);

    // Open DB and prepare OpenMLS provider
    if let Err(e) = session.set_provider() {
        state.set_flash(format!("Login failed: {e}"));
        return Action::Back;
    }

    // Init OpenMLS
    // TODO remove unwrap when possible
    let _ = mls::identity::init_openmls(
        session.db_key().unwrap(),
        session.user_id(),
        session.device_id().unwrap(),
        session.provider().unwrap(),
        is_new_device,
    );

    // Everything went good -> login successful and proceed to the "Logged In" menu screen
    state.set_flash("Login successful!");

    state.session = Some(session);
    Action::Replace(Screen::MenuLoggedIn)
}

// Create group screen, needs a session
fn create_group(state: &mut AppState) -> Action {
    let session = match state.session.as_ref() {
        Some(s) => s,
        None => {
            state.set_flash("Not logged in.");
            return Action::Back;
        }
    };

    let group_name = match prompt::group_name() {
        Ok(group_name) => group_name,
        Err(e) => {
            state.set_flash(format!("Could not read group name: {e}"));
            return Action::Back;
        }
    };

    let username = match prompt::invite_username() {
        Ok(username) => username,
        Err(e) => {
            state.set_flash(format!("Could not read username: {e}"));
            return Action::Back;
        }
    };

    // TODO remove unwrap when possible
    match mls::identity::init_group(
        session.db_key().unwrap(),
        session.user_id(),
        session.device_id().unwrap(),
        session.provider().unwrap(),
        &username,
        &group_name,
    ) {
        Ok(_) => (),
        Err(e) => {
            state.set_flash(format!("Could not initialize group : {e}"));
            return Action::Back;
        }
    }

    state.set_flash(format!("Creating group and inviting: {username}"));
    Action::Back
}

// Browse groups screen, needs a session
fn browse_groups(state: &mut AppState) -> Action {
    let session = match state.session.as_ref() {
        Some(s) => s,
        None => {
            state.set_flash("Not logged in.");
            return Action::Back;
        }
    };

    let groups: Vec<(openmls::prelude::GroupId, String)> =
        storage::database::retrieve_groups(session.db_key().unwrap(), session.user_id()).unwrap();

    prompt::browse_groups(groups);
    Action::Back
}

// One-shot helpers, no navigation change
fn add_friend() {
    if let Err(e) = transport::http::test_session() {
        eprintln!("An error occured : {e}");
    }
    println!("Your request has been sent.");
}

// Fetch welcome for current session
fn fetch_welcome(state: &AppState) {
    let session = match state.session.as_ref() {
        Some(s) => s,
        None => {
            eprintln!("Not logged in.");
            return;
        }
    };

    // TODO remove unwrap when possible
    match mls::identity::fetch_welcome(
        session.db_key().unwrap(),
        session.user_id(),
        session.provider().unwrap(),
    ) {
        Ok(_) => (),
        Err(e) => eprintln!("Error fetching welcomes : {e}"),
    };
}

// Test current session token
fn test_session() {
    if let Err(e) = transport::http::test_session() {
        eprintln!("Not autorized (no Session token) : {e}");
        return;
    }

    println!("Your session is valid.");
}
