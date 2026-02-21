#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub(crate) author: String,
    pub(crate) content: String,
    pub(crate) timestamp: String, // placeholder: simple string
}

#[derive(Debug)]
pub struct Chat {
    pub(crate) groupname: String,
    pub(crate) username: String,

    pub(crate) members: Vec<String>,

    // Message history
    pub(crate) messages: Vec<ChatMessage>,

    pub(crate) input: String,

    // scroll: how many lines we "skip" from the bottom
    pub(crate) scroll_from_bottom: u16,
}

impl Chat {
    pub fn new(groupname: &str, username: &str) -> Self {
        Self {
            groupname: groupname.to_string(),
            username: username.to_string(),
            messages: Vec::new(),
            input: String::new(),
            scroll_from_bottom: 0,
            // TODO: replace with real user list from server/state
            members: vec!["Alice".to_string(), "Bob".to_string(), "Me".to_string()],
        }
    }

    pub fn push_message(&mut self, author: String, content: String) {
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
        self.scroll_from_bottom = self.scroll_from_bottom.saturating_add(8);
    }

    pub fn page_down(&mut self) {
        self.scroll_from_bottom = self.scroll_from_bottom.saturating_sub(8);
    }
}

/*

EXEMPLE CHAT :

Self {
    groupname: "Room: general".to_string(),
    username: "Me".to_string(),
    messages: vec![
        ChatMessage {
            author: "System".to_string(),
            content: "Welcome to the chatroom (UI skeleton).".to_string(),
            timestamp: "00:00".to_string(),
        },
        ChatMessage {
            author: "Alice".to_string(),
            content: "Hi!".to_string(),
            timestamp: "00:01".to_string(),
        },
    ],
    input: String::new(),
    scroll_from_bottom: 0,
    members: vec!["Alice".to_string(), "Bob".to_string(), "Me".to_string()],
}

*/
