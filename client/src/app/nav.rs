use crate::{
    auth, mls, storage, transport,
    ui::{
        cli::{choice, prompt},
        tui,
    },
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
    FlowAddFriend,
    FlowCreateGroup,
    MenuBrowseGroups,
    Chatroom,
}

impl Screen {
    fn handle(self, state: &mut AppState) -> Action {
        match self {
            Screen::MenuLoggedOut => logged_out_menu(),
            Screen::FlowSignup => signup(state),
            Screen::FlowLogin => login(state),
            Screen::MenuLoggedIn => logged_in_menu(state),
            Screen::FlowAddFriend => add_friend(state),
            Screen::FlowCreateGroup => create_group(state),
            Screen::MenuBrowseGroups => browse_groups(state),
            Screen::Chatroom => chatroom(state),
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

// Display the app name and version on top of screen
fn display_header(state: &AppState) {
    println!("{} - {}", state.name(), state.version());
    println!("");
}

// Main router, all navigation is decided here
pub fn run(state: &mut AppState) {
    // Screen stack, top is current screen
    // Allow to easily return to the previous screen
    let mut stack = vec![Screen::MenuLoggedOut];
    let mut new_screen = true;

    loop {
        // Get current screen from stack top
        let screen = match stack.last().copied() {
            Some(s) => s,
            None => break,
        };

        if new_screen {
            clear_terminal();
            display_header(state);
            state.show_action_msg();
        }

        // Display the screen on terminal
        // And retrieve the user action
        let action = screen.handle(state);

        new_screen = false;

        // Apply the user action
        // Change the screen stack depending of what the user choosed
        match action {
            Action::Back => {
                stack.pop();
                if stack.is_empty() {
                    break;
                }
                new_screen = true;
            }
            Action::Quit => break,
            Action::Stay => {
                if state.has_action_msg() {
                    new_screen = true;
                }
            }
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

// Signup screen, then go back
fn signup(state: &mut AppState) -> Action {
    // Ask the user for his username and password
    let (username, password) = match prompt::signup() {
        Ok((username, password)) => (username, password),
        Err(e) => {
            state.set_action_msg(format!("Signup failed: {e}"));
            return Action::Back;
        }
    };

    // Try to register with OPAQUE
    match auth::opaque::register(&username, &password) {
        Ok(_) => state.set_action_msg("Registration successful!"),
        Err(e) => state.set_action_msg(format!("Registration failed: {e}")),
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
            state.set_action_msg(format!("Login failed: {e}"));
            return Action::Back;
        }
    };

    // Retrieve login results and create new session with it if successful
    let mut session = match auth::opaque::login(&username, &password) {
        Ok(login_result) => Session::new(login_result),
        Err(e) => {
            state.set_action_msg(format!("Login failed: {e}"));
            return Action::Back;
        }
    };

    // Init local storage and get final token (session token)
    let (device_id, db_key, is_new_device) =
        match storage::database::init_device_storage(session.user_id(), session.export_key()) {
            Ok(result) => result,
            Err(e) => {
                state.set_action_msg(format!("Device storage initialization failed: {e}"));
                return Action::Back;
            }
        };

    // Save device and storage info in the session
    session.set_device_id(&device_id);
    session.set_db_key(db_key);

    // Open DB and prepare OpenMLS provider
    if let Err(e) = session.set_provider() {
        state.set_action_msg(format!("Login failed: {e}"));
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
    state.set_action_msg("Login successful!");

    state.set_session(Some(session));
    Action::Replace(Screen::MenuLoggedIn)
}

// Main menu when logged in, each choice goes to a screen or runs once
fn logged_in_menu(state: &mut AppState) -> Action {
    match choice::prompt_logged_in() {
        Some(choice::LoggedInChoice::AddFriend) => {
            add_friend(state);
            Action::Stay
        }
        Some(choice::LoggedInChoice::CreateGroup) => Action::Push(Screen::FlowCreateGroup),
        Some(choice::LoggedInChoice::BrowseGroups) => Action::Push(Screen::MenuBrowseGroups),
        Some(choice::LoggedInChoice::FetchWelcome) => {
            fetch_welcome(state);
            Action::Stay
        }
        Some(choice::LoggedInChoice::TestSession) => {
            test_session(state);
            Action::Stay
        }
        Some(choice::LoggedInChoice::Logout) => {
            state.set_session(None);
            state.set_action_msg("Logged out.");
            Action::Back
        }
        None => Action::Back,
    }
}

// Add a new friend
fn add_friend(state: &mut AppState) -> Action {
    let user_to_add = match prompt::invite_username() {
        Ok(user_to_add) => user_to_add,
        Err(e) => {
            state.set_action_msg(format!("Could not read username: {e}"));
            return Action::Back;
        }
    };

    // TODO : call server route to send friend request

    state.set_action_msg(format!(
        "Your friend request to '{}' has been sent.",
        user_to_add
    ));
    Action::Back
}

// Create group screen, needs a session
fn create_group(state: &mut AppState) -> Action {
    let session = match state.session() {
        Some(s) => s,
        None => {
            state.set_action_msg("Not logged in.");
            return Action::Back;
        }
    };

    let group_name = match prompt::group_name() {
        Ok(group_name) => group_name,
        Err(e) => {
            state.set_action_msg(format!("Could not read group name: {e}"));
            return Action::Back;
        }
    };

    // TODO : Allow multiple user invitation, depending on future friendships
    let username = match prompt::invite_username() {
        Ok(username) => username,
        Err(e) => {
            state.set_action_msg(format!("Could not read username: {e}"));
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
            state.set_action_msg(format!("Could not initialize group : {e}"));
            return Action::Back;
        }
    }

    state.set_action_msg(format!("New group '{}' created.", group_name));
    Action::Back
}

// Browse groups screen, needs a session
fn browse_groups(state: &mut AppState) -> Action {
    let session = match state.session() {
        Some(s) => s,
        None => {
            state.set_action_msg("Not logged in.");
            return Action::Back;
        }
    };

    let groups: Vec<(openmls::prelude::GroupId, String)> =
        storage::database::retrieve_groups(session.db_key().unwrap(), session.user_id()).unwrap();

    // Display the groups, and let the user choose one
    let selected_group = match prompt::browse_groups(groups) {
        Some(g) => g,
        None => return Action::Back,
    };

    // Store the selected group in the app state
    state.set_selected_group(selected_group);

    // Back to Browse Groups after exiting the TUI (Ctrl+C / Esc)
    Action::Push(Screen::Chatroom)
}

// Display the group (chatroom) previously selected and stored in the app state
fn chatroom(state: &mut AppState) -> Action {
    let session = match state.session() {
        Some(s) => s,
        None => {
            state.set_action_msg("Not logged in.");
            return Action::Back;
        }
    };

    // Retrieve the group
    let group = match state.selected_group() {
        Some(g) => g,
        None => {
            state.set_action_msg("No group selected.");
            return Action::Back;
        }
    };

    // Create a new chat with the desired group
    let mut chat = tui::chat::Chat::new(&group.name, session.username());
    if let Err(e) = tui::driver::run(&mut chat) {
        state.set_action_msg(format!("Chat error: {e}"));
    }

    // When user exit, back to the group list (BrowseGroup)
    Action::Back
}

////////////////////
// DEBUG FUNCTIONS :

// Fetch welcome for current session
fn fetch_welcome(state: &mut AppState) {
    let session = match state.session() {
        Some(s) => s,
        None => {
            state.set_action_msg("Not logged in.");
            return;
        }
    };

    // TODO remove unwrap when possible
    match mls::identity::fetch_welcome(
        session.db_key().unwrap(),
        session.user_id(),
        session.provider().unwrap(),
    ) {
        Ok(_) => state.set_action_msg("Welcomes have been fetched."),
        Err(e) => state.set_action_msg(format!("Error fetching welcomes : {e}")),
    };
}

// Test current session token
fn test_session(state: &mut AppState) {
    if let Err(e) = transport::http::test_session() {
        state.set_action_msg(format!("Not autorized (no Session token) : {e}"));
        return;
    }

    state.set_action_msg("Your session is valid.");
}
