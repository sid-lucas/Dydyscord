mod auth;
mod config;
mod error;
mod mls;
mod storage;
mod transport;
mod ui;

use auth::session::AppState;
use auth::session::Session;
use storage::database;
use ui::choice;

use crate::transport::http;

fn main() {
    let mut appstate = AppState::LoggedOut;

    loop {
        let next = match appstate {
            AppState::LoggedOut => handle_logged_out(),
            AppState::LoggedIn(session) => handle_logged_in(session),
        };

        match next {
            Some(state) => appstate = state,
            None => break,
        }
    }
}

fn handle_logged_out() -> Option<AppState> {
    match choice::prompt_logged_out() {
        choice::LoggedOutChoice::Signup => signup(),
        choice::LoggedOutChoice::Login => login(),
        choice::LoggedOutChoice::Quit => None,
    }
}

fn handle_logged_in(session: Session) -> Option<AppState> {
    match choice::prompt_logged_in() {
        choice::LoggedInChoice::AddFriend => add_friend(session),
        choice::LoggedInChoice::TestSession => test_session(session),
        choice::LoggedInChoice::Logout => {
            drop(session);
            println!("Logged out.");
            Some(AppState::LoggedOut)
        }
    }
}

fn signup() -> Option<AppState> {
    let (username, password) = match ui::prompt::signup() {
        Ok((username, password)) => (username, password),
        Err(e) => {
            eprintln!("Signup failed: {e}");
            return Some(AppState::LoggedOut);
        }
    };

    match auth::opaque::register(&username, &password) {
        Ok(_) => {
            println!("Registration successful!");
            Some(AppState::LoggedOut)
        }
        Err(e) => {
            eprintln!("Registration failed: {e}");
            Some(AppState::LoggedOut)
        }
    }
}

fn login() -> Option<AppState> {
    // OPAQUE handshake with the server and retrieval of JWT Auth
    let (username, password) = match ui::prompt::login() {
        Ok((username, password)) => (username, password),
        Err(e) => {
            eprintln!("Login failed: {e}");
            return Some(AppState::LoggedOut);
        }
    };

    // Add OPAQUE login information to the session
    let mut session = match auth::opaque::login(&username, &password) {
        Ok(login_result) => Session::new(login_result),
        Err(e) => {
            eprintln!("Login failed: {e}");
            return Some(AppState::LoggedOut);
        }
    };

    // Initialize local storage files and retrieve Session token
    let (device_id, db_key, is_new_device) =
        match database::init_device_storage(session.user_id(), session.export_key()) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Device storage initialization failed: {e}");
                return Some(AppState::LoggedOut);
            }
        };

    // Add device/storage initialization information to the session
    session.set_device_id(&device_id);
    session.set_db_key(db_key);

    // Open the db connection and prepare the OpenMLS provider
    if let Err(e) = session.set_provider() {
        eprintln!("Login failed: {e}");
        return Some(AppState::LoggedOut);
    }

    // Initialize OpenMLS
    let _ = mls::identity::init_openmls(is_new_device, device_id);

    println!("Login successful!");
    Some(AppState::LoggedIn(session))
}

fn add_friend(session: Session) -> Option<AppState> {
    if let Err(e) = http::test_session() {
        eprintln!("An error occured : {e}");
    }
    println!("Your request has been sent.");
    Some(AppState::LoggedIn(session))
}

fn test_session(session: Session) -> Option<AppState> {
    if let Err(e) = http::test_session() {
        eprintln!("Not autorized (no Session token) : {e}");
        return Some(AppState::LoggedOut);
    }
    println!("Your session is valid.");
    Some(AppState::LoggedIn(session))
}
