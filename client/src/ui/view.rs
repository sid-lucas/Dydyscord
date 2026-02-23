pub mod chat;
pub mod info;

use crate::ui::{form::view::FormState, menu::view::MenuState};
pub use chat::{Chat, ChatMessage, ChatRoom, ChatState};
pub use info::InfoState;

pub enum View {
    Menu(MenuState),
    Form(FormState),
    Info(InfoState),
    Chat(ChatState),
}
