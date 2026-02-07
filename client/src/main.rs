use inquire_derive::Selectable;
use std::fmt;

use crate::{mls::storage::get_or_create_db_key, opaque::auth::login, session::Session};

mod api;
mod error;
mod mls;
mod opaque;
mod session;

enum Appstate {
    LoggedOut,
    LoggedIn(session::Session),
}

#[derive(Debug, Copy, Clone, Selectable)]
enum LoggedOutChoice {
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
enum LoggedInChoice {
    Test,
    Logout,
}

impl fmt::Display for LoggedInChoice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoggedInChoice::Test => write!(f, "Test"),
            LoggedInChoice::Logout => write!(f, "Log Out"),
        }
    }
}

// TODO : faire un fichier contenant tout les variables const (notamment celle récup du .env)
// TODO : et peut etre utiliser OnceCell sur ces variables... a discuter

// TODO : PEUT ETRE SEPARER TOUT CA DANS UN FICHIER cli.rs ET JUSTE GARDER LE MAIN() ET LES MOD

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
    }
}

fn run() -> Result<(), error::ClientError> {
    let mut appstate = Appstate::LoggedOut;

    loop {
        match appstate {
            Appstate::LoggedOut => {
                let choice = LoggedOutChoice::select("Choose an option:")
                    .prompt()
                    .expect("An error occurred");

                match choice {
                    LoggedOutChoice::Login => match opaque::auth::login() {
                        Ok(login_result) => {
                            // Ajout des informations de login OPAQUE dans la session
                            let mut session = Session::new(login_result);

                            // Reconcile + récupère si le device est reconnu avant potentielle init de la db
                            let new_device =
                                !session::reconcile_device_storage(&session.user_id.to_string());

                            // Récupèration/Création de la clé de chiffrement de la db
                            let db_key = get_or_create_db_key(
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
                            appstate = Appstate::LoggedIn(session);
                        }
                        Err(e) => {
                            eprintln!("Login failed: {e}");
                            continue;
                        }
                    },
                    LoggedOutChoice::Signup => match opaque::auth::register() {
                        Ok(_) => println!("Registration successful!"),
                        Err(e) => {
                            eprintln!("Registration failed: {e}");
                            continue;
                        }
                    },
                    LoggedOutChoice::Quit => break,
                }
            }
            Appstate::LoggedIn(ref session) => {
                let choice = LoggedInChoice::select("Choose an option:")
                    .prompt()
                    .expect("An error occurred");

                match choice {
                    LoggedInChoice::Test => {
                        // utiliser session ici
                        println!("test");
                    }
                    LoggedInChoice::Logout => {
                        // TODO CLEAR session
                        println!("need to clear the actual session then proceed.");
                        appstate = Appstate::LoggedOut;
                    }
                }
            }
        }
    }

    Ok(())
}
