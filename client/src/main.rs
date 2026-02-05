use inquire_derive::Selectable;
use std::fmt;
mod api;
mod error;
mod mls;
mod opaque;

#[derive(Debug, Copy, Clone, Selectable)]
enum Choice {
    Register,
    Login,
    Test,
}

impl fmt::Display for Choice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Choice::Register => write!(f, "Sign Up"),
            Choice::Login => write!(f, "Log In"),
            Choice::Test => write!(f, "Test"),
        }
    }
}

fn main() {
    loop {
        let answer = Choice::select("Choose an option:")
            .prompt()
            .expect("An error occurred");

        match answer {
            Choice::Register => match opaque::auth::register() {
                Ok(_) => println!("Registration successful!"),
                Err(e) => eprintln!("Registration failed: {e}"),
            },
            Choice::Login => match opaque::auth::login() {
                Ok(_) => {
                    println!("Login successful!");
                }
                Err(e) => eprintln!("Login failed: {e}"),
            },
            Choice::Test => match mls::test::test() {
                Ok(_) => {
                    println!("TEST OK");
                }
                Err(e) => eprintln!("TEST PAS OK: {e}"),
            },
        }
    }
}
