#[derive(Clone, Debug)]
pub struct ChatMessage {
    // Who wrote the message.
    pub(crate) author: String,
    // Message content (raw text).
    pub(crate) content: String,
    // Placeholder timestamp; swap with real time later.
    pub(crate) timestamp: String, // placeholder: simple string
}

#[derive(Clone, Debug)]
pub struct ChatState {
    // Display name for the room.
    pub room_name: String,
    // Current user name.
    pub user_name: String,
    // User list in the room.
    pub users: Vec<String>,
    // Message history.
    pub messages: Vec<ChatMessage>,
}

#[derive(Clone, Debug)]
pub struct ChatRoom {
    // Short room name (used in menu list).
    pub name: String,
    // Full chat UI state for this room.
    pub chat: Chat,
}

impl ChatRoom {
    pub fn new(
        name: impl Into<String>,
        user_name: impl Into<String>,
        users: Vec<String>,
        messages: Vec<ChatMessage>,
    ) -> Self {
        // Build a Chat struct from raw state; convenient for seeding data.
        let name = name.into();
        let chat = Chat::from_state(ChatState {
            room_name: format!("Room: {}", name),
            user_name: user_name.into(),
            users,
            messages,
        });

        Self { name, chat }
    }
}

#[derive(Clone, Debug)]
pub struct Chat {
    // Display name shown in the header.
    pub(crate) room_name: String,
    // Current user (used to highlight own messages).
    pub(crate) user_name: String,

    // Stored message history.
    pub(crate) messages: Vec<ChatMessage>,
    // Current input text.
    pub(crate) input: String,

    // scroll: how many lines we "skip" from the bottom
    // We keep scroll-from-bottom so new messages stay visible.
    pub(crate) scroll_from_bottom: u16,

    // optional: user list
    // User list for the sidebar.
    pub(crate) users: Vec<String>,
}

impl Chat {
    pub fn from_state(state: ChatState) -> Self {
        // Convert an external state into internal UI state.
        Self {
            room_name: state.room_name,
            user_name: state.user_name,
            users: state.users,
            messages: state.messages,
            input: String::new(),
            scroll_from_bottom: 0,
        }
    }

    pub fn push_message(&mut self, author: String, content: String) {
        // Append a message if it's not empty; also snap scroll to bottom.
        let content = content.trim().to_string();
        if content.is_empty() {
            return;
        }
        self.messages.push(ChatMessage {
            author,
            content,
            timestamp: Self::fake_time(),
        });
        // When sending a message, jump back to the bottom
        self.scroll_from_bottom = 0;
    }

    fn fake_time() -> String {
        // Simple placeholder so messages look "real" in the UI.
        // TODO: replace with real time (chrono/time)
        "12:34".to_string()
    }

    pub fn scroll_up(&mut self) {
        // ↑ : go up => increase scroll_from_bottom
        self.scroll_from_bottom = self.scroll_from_bottom.saturating_add(1);
    }

    pub fn scroll_down(&mut self) {
        // ↓ : go down => decrease scroll_from_bottom
        self.scroll_from_bottom = self.scroll_from_bottom.saturating_sub(1);
    }

    pub fn page_up(&mut self) {
        // Larger jump for fast scrolling.
        self.scroll_from_bottom = self.scroll_from_bottom.saturating_add(8);
    }

    pub fn page_down(&mut self) {
        // Larger jump for fast scrolling.
        self.scroll_from_bottom = self.scroll_from_bottom.saturating_sub(8);
    }
}
