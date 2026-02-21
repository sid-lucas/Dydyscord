pub mod chat;
pub mod form;
pub mod info;
pub mod menu;

pub use chat::{Chat, ChatMessage, ChatRoom, ChatState};
pub use form::{FormKind, FormState, LoginField, LoginFormState, SignupField, SignupFormState};
pub use info::InfoState;
pub use menu::{MenuAction, MenuEntry, MenuFrame, MenuPageKind, MenuState};

pub enum View {
    Menu(MenuState),
    Form(FormState),
    Info(InfoState),
    Chat(ChatState),
}
