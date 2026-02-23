pub mod chat;
pub mod info;
pub mod menu;

use crate::ui::form::view::FormState;
pub use chat::{Chat, ChatMessage, ChatRoom, ChatState};
pub use info::InfoState;
pub use menu::{MenuAction, MenuEntry, MenuFrame, MenuPageKind, MenuState};

pub enum View {
    Menu(MenuState),
    Form(FormState),
    Info(InfoState),
    Chat(ChatState),
}
