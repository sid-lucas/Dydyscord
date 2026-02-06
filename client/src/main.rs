use inquire_derive::Selectable;
use std::fmt;

use crate::session::Session;

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
                            // OPAQUE
                            let session = Session::new(login_result);

                            // OpenMLS
                            if session::device_exists(
                                &session.user_id.to_string(),
                                &session.device_id,
                            ) {
                                // Faire appel au serveur genre /create/device
                                // pour créer un nouveau device dans la bdd lié à l'utilisateur loggé
                                // et retourner "device_id" à l'utilisateur
                                // device_id servira à stocker la db_key dans le keystore
                                // device_id sera aussi présent dans device.db du client (normalement)
                            } else {
                                //
                                // La c'est si le device est reconnu (a deja fait l'initialisation OpenMLS)
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
