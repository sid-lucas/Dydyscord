use crate::ui::{form::view::FormState, menu::view::MenuState};
pub use crate::ui::chat::view::{Chat, ChatMessage, ChatRoom, ChatState};
pub use crate::ui::info::view::InfoState;

pub enum View {
    Menu(MenuState),
    Form(FormState),
    Info(InfoState),
    Chat(ChatState),
}
