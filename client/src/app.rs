use crate::{
    choice, error,
    mls::storage,
    opaque,
    session::{self, Session},
};

enum AppState {
    LoggedOut,
    LoggedIn(Session),
}

pub fn run() -> Result<(), error::ClientError> {
    let mut appstate = AppState::LoggedOut;

    loop {
        let next = match appstate {
            AppState::LoggedOut => handle_logged_out()?,
            AppState::LoggedIn(session) => handle_logged_in(session)?,
        };

        match next {
            Some(state) => appstate = state,
            None => break,
        }
    }

    Ok(())
}

fn handle_logged_out() -> Result<Option<AppState>, error::ClientError> {
    match choice::prompt_logged_out() {
        choice::LoggedOutChoice::Login => match opaque::auth::login() {
            Ok(login_result) => {
                // Ajout des informations de login OPAQUE dans la session
                let mut session = Session::new(login_result);

                // Reconcile + récupère si le device est reconnu avant potentielle init de la db
                let new_device = !session::reconcile_device_storage(&session.user_id.to_string());

                // Récupèration/Création de la clé de chiffrement de la db
                let db_key = storage::get_or_create_db_key(
                    &session.user_id.to_string(),
                    &session.export_key,
                )
                .unwrap();

                // Ouverture de la connexion de la db et préparation du provider OpenMLS
                session.set_provider(&db_key)?;

                if new_device {
                    // Faire appel au serveur genre /create/device
                    // pour créer un nouveau device dans la bdd lié à l'utilisateur loggé
                    // et retourner "device_id" à l'utilisateur
                    // device_id sera présent dans device.db du client (normalement)
                    // car je crois qu'il sera important quand on fera des requêtes au serveur de lui
                    // montrer quel device du compte on utilise pour la création de groupe ou autre.

                    // TODO : CREER LES TYPES OPENMLS NECESSAIRES ET STOCKER DANS LA DB LOCALE
                    println!("New device detected.")
                } else {
                    //
                    // La c'est si le device est reconnu (a deja fait l'initialisation OpenMLS)

                    // TODO : LIRE LES TYPES DE LA DB LOCALE
                    println!("Retrieved device information.")
                }

                println!("Login successful!");
                Ok(Some(AppState::LoggedIn(session)))
            }
            Err(e) => {
                eprintln!("Login failed: {e}");
                Ok(Some(AppState::LoggedOut))
            }
        },
        choice::LoggedOutChoice::Signup => match opaque::auth::register() {
            Ok(_) => {
                println!("Registration successful!");
                Ok(Some(AppState::LoggedOut))
            }
            Err(e) => {
                eprintln!("Registration failed: {e}");
                Ok(Some(AppState::LoggedOut))
            }
        },
        choice::LoggedOutChoice::Quit => Ok(None),
    }
}

fn handle_logged_in(session: Session) -> Result<Option<AppState>, error::ClientError> {
    match choice::prompt_logged_in() {
        choice::LoggedInChoice::Test => {
            // utiliser session ici
            println!("test");
            Ok(Some(AppState::LoggedIn(session)))
        }
        choice::LoggedInChoice::Logout => {
            // TODO CLEAR session
            println!("need to clear the actual session then proceed.");
            Ok(Some(AppState::LoggedOut))
        }
    }
}
