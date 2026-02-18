use crate::auth::session::Session;
use crate::config::constant;

pub struct AppState {
    name: String,
    version: String,
    session: Option<Session>,   // Session of the logged in user
    action_msg: Option<String>, // Allow to keep the message to display it on next screen
}

impl AppState {
    pub fn new() -> Self {
        Self {
            name: constant::APP_NAME.to_string(),
            version: constant::APP_VERSION.to_string(),
            session: None,
            action_msg: None,
        }
    }

    // Getter
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn version(&self) -> &str {
        self.version.as_str()
    }

    pub fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    // Setter
    pub fn set_session(&mut self, session: Option<Session>) {
        self.session = session;
    }

    pub fn set_action_msg(&mut self, msg: impl Into<String>) {
        self.action_msg = Some(msg.into());
    }

    pub fn has_action_msg(&self) -> bool {
        self.action_msg.is_some()
    }

    pub fn show_action_msg(&mut self) {
        if let Some(msg) = self.action_msg.take() {
            println!("{msg}");
            println!("");
        }
    }
}
