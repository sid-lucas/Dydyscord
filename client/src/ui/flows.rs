use secrecy::SecretSlice;

use crate::core::{auth as core_auth, groups as core_groups};
use crate::transport;
use crate::ui::app::{App, BlockingAction, MenuState, View};
use crate::ui::chat::ChatRoom;
use crate::ui::cli::prompt;

pub fn run(app: &mut App, action: BlockingAction) {
    match action {
        BlockingAction::Signup => signup(app),
        BlockingAction::Login => login(app),
        BlockingAction::AddFriend => add_friend(app),
        BlockingAction::CreateGroup => create_group(app),
        BlockingAction::BrowseGroups => browse_groups(app),
        BlockingAction::FetchWelcome => fetch_welcome(app),
        BlockingAction::TestSession => test_session(app),
    }
}

// Signup screen, then go back
fn signup(app: &mut App) {
    // Ask the user for his username and password
    let (username, password) = match prompt::signup() {
        Ok((username, password)) => (username, password),
        Err(e) => {
            app.set_action_msg(format!("Signup failed: {e}"));
            return;
        }
    };

    let password = SecretSlice::from(password.into_bytes());

    // Try to register with OPAQUE
    match core_auth::signup(&username, &password) {
        Ok(_) => app.set_action_msg("Registration successful!"),
        Err(e) => app.set_action_msg(format!("Registration failed: {e}")),
    }
}

// Login screen, on success go to MenuLoggedIn
fn login(app: &mut App) {
    // OPAQUE login with server to get intermediate token (auth token)
    let (username, password) = match prompt::login() {
        Ok((username, password)) => (username, password),
        Err(e) => {
            app.set_action_msg(format!("Login failed: {e}"));
            return;
        }
    };

    let password = SecretSlice::from(password.into_bytes());

    // Retrieve login results and create new session with it if successful
    let session = match core_auth::login(&username, &password) {
        Ok(session) => session,
        Err(e) => {
            app.set_action_msg(format!("Login failed: {e}"));
            return;
        }
    };

    // Everything went good -> login successful and proceed to the "Logged In" menu screen
    app.set_action_msg("Login successful!");

    app.set_session(Some(session));
    app.ui.view = View::Menu(MenuState::authed_root());

    // Spawn the websocket in background
    transport::ws::start_background();
}

// Add a new friend
fn add_friend(app: &mut App) {
    let user_to_add = match prompt::invite_username() {
        Ok(user_to_add) => user_to_add,
        Err(e) => {
            app.set_action_msg(format!("Could not read username: {e}"));
            return;
        }
    };

    // TODO : call server route to send friend request
    if let Err(e) = core_groups::add_friend(&user_to_add) {
        app.set_action_msg(format!("Could not add friend: {e}"));
        return;
    }

    app.set_action_msg(format!(
        "Your friend request to '{}' has been sent.",
        user_to_add
    ));
}

// Create group screen, needs a session
fn create_group(app: &mut App) {
    let session = match app.session() {
        Some(s) => s,
        None => {
            app.set_action_msg("Not logged in.");
            return;
        }
    };

    let group_name = match prompt::group_name() {
        Ok(group_name) => group_name,
        Err(e) => {
            app.set_action_msg(format!("Could not read group name: {e}"));
            return;
        }
    };

    // TODO : Allow multiple user invitation, depending on future friendships
    let username = match prompt::invite_username() {
        Ok(username) => username,
        Err(e) => {
            app.set_action_msg(format!("Could not read username: {e}"));
            return;
        }
    };

    // TODO remove unwrap when possible
    match core_groups::create_group(session, &group_name, &username) {
        Ok(_) => (),
        Err(e) => {
            app.set_action_msg(format!("Could not initialize group : {e}"));
            return;
        }
    }

    app.set_action_msg(format!("New group '{}' created.", group_name));
}

// Browse groups screen, needs a session
fn browse_groups(app: &mut App) {
    let session = match app.session() {
        Some(s) => s,
        None => {
            app.set_action_msg("Not logged in.");
            return;
        }
    };

    let groups = match core_groups::browse_groups(session) {
        Ok(groups) => groups,
        Err(e) => {
            app.set_action_msg(format!("Could not load groups: {e}"));
            return;
        }
    };

    // Display the groups, and let the user choose one
    let selected_group = match prompt::browse_groups(groups) {
        Some(g) => g,
        None => {
            app.set_action_msg("You're not part of any groups.");
            return;
        }
    };

    // Store the selected group in the app state
    app.set_selected_group(selected_group);

    chatroom(app);
}

// Display the group (chatroom) previously selected and stored in the app state
fn chatroom(app: &mut App) {
    let session = match app.session() {
        Some(s) => s,
        None => {
            app.set_action_msg("Not logged in.");
            return;
        }
    };

    // Retrieve the group
    let group = match app.selected_group() {
        Some(g) => g,
        None => {
            app.set_action_msg("No group selected.");
            return;
        }
    };

    // Create a new chat with the desired group
    let room = ChatRoom::new(&group.name, session.username(), Vec::new(), Vec::new());
    app.data.rooms = Some(vec![room]);
    app.ui.view = View::Chat { room_index: 0 };
}

////////////////////
// DEBUG FUNCTIONS :

// Fetch welcome for current session
fn fetch_welcome(app: &mut App) {
    let session = match app.session() {
        Some(s) => s,
        None => {
            app.set_action_msg("Not logged in.");
            return;
        }
    };

    // TODO remove unwrap when possible
    match core_groups::fetch_welcome(session) {
        Ok(_) => app.set_action_msg("Welcomes have been fetched."),
        Err(e) => app.set_action_msg(format!("Error fetching welcomes : {e}")),
    };
}

// Test current session token
fn test_session(app: &mut App) {
    if let Err(e) = core_groups::test_session() {
        app.set_action_msg(format!("Not autorized (no Session token) : {e}"));
        return;
    }

    app.set_action_msg("Your session is valid.");
}
