mod auth;
mod config;
mod storage;
mod transport;
mod ui;
use auth::session::AppState;
use auth::session::Session;
use storage::database;
use transport::http;
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
            // TODO CLEAR session
            println!("need to clear the actual session then proceed.");
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
    let (username, password) = match ui::prompt::login() {
        Ok((username, password)) => (username, password),
        Err(e) => {
            eprintln!("Login failed: {e}");
            return None;
        }
    };

    match auth::opaque::login(&username, &password) {
        Ok(login_result) => {
            // Ajout des informations de login OPAQUE dans la session
            let mut session = Session::new(login_result);

            // Reconcile + récupère si le device est reconnu avant potentielle init de la db
            let new_device = !database::reconcile_device_storage(&session.user_id.to_string());

            // Récupèration/Création de la clé de chiffrement de la db
            let db_key =
                database::get_db_key(&session.user_id.to_string(), &session.export_key).unwrap();

            // Ouverture de la connexion de la db et préparation du provider OpenMLS
            session.set_provider(&db_key, &session.user_id.to_string())?;

            if new_device {
                // TODO : CREER LES TYPES OPENMLS NECESSAIRES ET STOCKER DANS LA DB LOCALE
                let device_id = http::new_device()
                    .map_err(|e| {
                        eprintln!("Failed to create new device: {e}");
                    })
                    .ok()?;

                println!("New device detected: {device_id}");
            } else {
                //
                // La c'est si le device est reconnu (a deja fait l'initialisation OpenMLS)

                // TODO : LIRE LES TYPES DE LA DB LOCALE
                println!("Retrieved device information.")
            }

            println!("Login successful!");
            Some(AppState::LoggedIn(session))
        }
        Err(e) => {
            eprintln!("Login failed: {e}");
            Some(AppState::LoggedOut)
        }
    }
}
