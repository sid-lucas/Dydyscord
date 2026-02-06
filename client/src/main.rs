use inquire_derive::Selectable;
use std::fmt;

use crate::opaque::auth::login;
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
    Logout,
    Test,
}

impl fmt::Display for LoggedInChoice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoggedInChoice::Logout => write!(f, "Log Out"),
            LoggedInChoice::Test => write!(f, "Test"),
        }
    }
}

// TODO : PEUT ETRE SEPARER TOUT CA DANS UN FICHIER cli.rs ET JUSTE GARDER LE MAIN() ET LES MOD

fn main() {
    if let Err(e) = run() {
        eprintln!("Erreur: {e}");
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
                    LoggedOutChoice::Login => {
                        let login_result = opaque::auth::login();
                        match login_result {
                            Ok(_) => {
                                println!("Login successful!");
                            }
                            Err(e) => eprintln!("Login failed: {e}"),
                        }
                        //let session = Session::new(login_result)?;
                        //appstate = Appstate::LoggedIn(session);
                    }
                    LoggedOutChoice::Signup => match opaque::auth::register() {
                        Ok(_) => println!("Registration successful!"),
                        Err(e) => eprintln!("Registration failed: {e}"),
                    },
                    LoggedOutChoice::Quit => break,
                }
            }
            Appstate::LoggedIn(ref session) => {
                let choice = LoggedInChoice::select("Choose an option:")
                    .prompt()
                    .expect("An error occurred");

                match choice {
                    LoggedInChoice::Logout => {
                        // TODO CLEAR session
                        println!("need to clear the actual session then proceed.");
                        appstate = Appstate::LoggedOut;
                    }
                    LoggedInChoice::Test => {
                        // utiliser session ici
                        println!("test");
                    }
                }
            }
        }
    }

    Ok(())
}
