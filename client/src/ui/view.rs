pub mod chat;
pub mod form;
pub mod info;
pub mod menu;

use form::{LoginFormState, SignupFormState};
use menu::MenuState;

pub use chat::{Chat, ChatMessage, ChatRoom, ChatState};
pub use form::{LoginField, LoginFormState, SignupField, SignupFormState};
pub use info::InfoState;
pub use menu::{MenuAction, MenuEntry, MenuFrame, MenuPageKind, MenuState};

pub enum View {
    // Menu with nested submenus.
    Menu(MenuState),
    // Simple login form.
    Login(LoginFormState),
    Signup(SignupFormState),
}
