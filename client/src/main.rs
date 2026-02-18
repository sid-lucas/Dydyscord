mod auth;
mod config;
mod error;
mod mls;
mod storage;
mod transport;
mod ui;

use auth::session::Session;
use storage::database;
use transport::http;
use ui::cli::{choice, prompt};

fn main() {
    loop {
        match choice::prompt_logged_out() {
            choice::LoggedOutChoice::Signup => signup(),
            choice::LoggedOutChoice::Login => {
                let session = match login() {
                    Some(session) => session,
                    None => continue,
                };
                handle_logged_in(session);
            }
            choice::LoggedOutChoice::Quit => break,
        };
    }
}

fn signup() {
    let (username, password) = match prompt::signup() {
        Ok((username, password)) => (username, password),
        Err(e) => {
            eprintln!("Signup failed: {e}");
            return;
        }
    };

    match auth::opaque::register(&username, &password) {
        Ok(_) => {
            println!("Registration successful!");
        }
        Err(e) => {
            eprintln!("Registration failed: {e}");
        }
    }
}

fn login() -> Option<Session> {
    // OPAQUE handshake with the server and retrieval of JWT Auth
    let (username, password) = match prompt::login() {
        Ok((username, password)) => (username, password),
        Err(e) => {
            eprintln!("Login failed: {e}");
            return None;
        }
    };

    // Add OPAQUE login information to the session
    let mut session = match auth::opaque::login(&username, &password) {
        Ok(login_result) => Session::new(login_result),
        Err(e) => {
            eprintln!("Login failed: {e}");
            return None;
        }
    };

    // Initialize local storage files and retrieve Session token
    let (device_id, db_key, is_new_device) =
        match database::init_device_storage(session.user_id(), session.export_key()) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Device storage initialization failed: {e}");
                return None;
            }
        };

    // Add device/storage initialization information to the session
    session.set_device_id(&device_id);
    session.set_db_key(db_key);

    // Open the db connection and prepare the OpenMLS provider
    if let Err(e) = session.set_provider() {
        eprintln!("Login failed: {e}");
        return None;
    }

    // Initialize OpenMLS
    // TODO : Use of "unwrap", change into something clean, even tho it can't be "None" at this point...
    let _ = mls::identity::init_openmls(
        session.db_key().unwrap(),
        session.user_id(),
        session.device_id().unwrap(),
        session.provider().unwrap(),
        is_new_device,
    );

    println!("Login successful!");

    Some(session)
}

fn handle_logged_in(session: Session) {
    loop {
        match choice::prompt_logged_in() {
            Some(choice::LoggedInChoice::AddFriend) => add_friend(),
            Some(choice::LoggedInChoice::CreateGroup) => create_group(&session),
            Some(choice::LoggedInChoice::BrowseGroups) => browse_groups(&session),
            Some(choice::LoggedInChoice::FetchWelcome) => fetch_welcome(&session),
            Some(choice::LoggedInChoice::TestSession) => test_session(),
            Some(choice::LoggedInChoice::Logout) => {
                drop(session);
                println!("Logged out.");
                break;
            }
            None => break, // Ctrl+C / ESC -> retour menu précédent
        };
    }
}

fn add_friend() {
    if let Err(e) = http::test_session() {
        eprintln!("An error occured : {e}");
    }
    println!("Your request has been sent.");
}

fn create_group(session: &Session) {
    let group_name = match prompt::group_name() {
        Ok(group_name) => group_name,
        Err(e) => {
            eprintln!("Could not read group name: {e}");
            return;
        }
    };

    let username = match prompt::invite_username() {
        Ok(username) => username,
        Err(e) => {
            eprintln!("Could not read username: {e}");
            return;
        }
    };

    // TODO : Change use of "unwrap", even tho provider cannot be "None" here...
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
            eprintln!("Could not initialize group : {e}");
            return;
        }
    }

    println!("Creating group and inviting: {username}");
}

fn browse_groups(session: &Session) {
    let groups: Vec<(openmls::prelude::GroupId, String)> =
        storage::database::retrieve_groups(session.db_key().unwrap(), session.user_id()).unwrap();

    prompt::browse_groups(groups);
}

fn fetch_welcome(session: &Session) {
    // TODO : Change use of "unwrap", even tho provider cannot be "None" here...
    match mls::identity::fetch_welcome(
        session.db_key().unwrap(),
        session.user_id(),
        session.provider().unwrap(),
    ) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error fetching welcomes : {e}")
        }
    };
}

fn test_session() {
    if let Err(e) = http::test_session() {
        eprintln!("Not autorized (no Session token) : {e}");
        return;
    }

    println!("Your session is valid.");
}
