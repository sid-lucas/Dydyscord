use crate::auth::session::Session;

pub struct AppState {
    pub session: Option<Session>, // Session of the logged in user
    pub flash: Option<String>, // Allow to keep the success/error message to display it on next screen
}

impl AppState {
    pub fn new() -> Self {
        Self {
            session: None,
            flash: None,
        }
    }

    pub fn set_flash(&mut self, msg: impl Into<String>) {
        self.flash = Some(msg.into());
    }

    pub fn take_flash(&mut self) -> Option<String> {
        self.flash.take()
    }

    pub fn show_flash(&mut self) {
        if let Some(msg) = self.flash.take() {
            println!("{msg}");
        }
    }
}
