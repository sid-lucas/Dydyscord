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
        choice::LoggedInChoice::Test => {
            // utiliser session ici
            println!("test");
            Some(AppState::LoggedIn(session))
        }
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
    // Handshake OPAQUE avec le serveur et récupèration du JWT Auth
    let (username, password) = match ui::prompt::login() {
        Ok((username, password)) => (username, password),
        Err(e) => {
            eprintln!("Login failed: {e}");
            return Some(AppState::LoggedOut);
        }
    };

    // Ajout des informations de login OPAQUE dans la session
    let mut session = match auth::opaque::login(&username, &password) {
        Ok(login_result) => Session::new(login_result),
        Err(e) => {
            eprintln!("Login failed: {e}");
            return Some(AppState::LoggedOut);
        }
    };

    // Initialisation des fichiers de storage local et récupèration du JWT Refresh
    let (device_id, db_key, is_new_device) =
        match database::init_device_storage(&session.user_id.to_string(), &session.export_key) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Device storage initialization failed: {e}");
                return Some(AppState::LoggedOut);
            }
        };

    // Ajout des informations de init device/storage dans la session
    // TODO

    // Ouverture de la connexion de la db et préparation du provider OpenMLS
    if let Err(e) = session.set_provider(&db_key, &session.user_id.to_string()) {
        eprintln!("Login failed: {e}");
        return Some(AppState::LoggedOut);
    }

    println!("Login successful!");
    Some(AppState::LoggedIn(session))
}
