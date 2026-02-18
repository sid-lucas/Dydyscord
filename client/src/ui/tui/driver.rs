use std::{
    io::{self, Stdout},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use super::{
    chat::{Chat, ChatState},
    draw,
};

pub fn run(state: ChatState) -> io::Result<()> {
    let mut terminal = init_terminal()?;
    let mut chat = Chat::from_state(state);
    let res = run_app(&mut terminal, &mut chat);
    restore_terminal(&mut terminal)?;
    res
}

// --- Terminal setup/restore ---

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

// --- Runtime loop ---

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, chat: &mut Chat) -> io::Result<()> {
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| draw::ui(f, chat))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if handle_key(chat, key) {
                    return Ok(()); // quit
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

// --- Event handling ---

fn handle_key(chat: &mut Chat, key: KeyEvent) -> bool {
    // Quit
    if key.code == KeyCode::Esc || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)) {
        return true;
    }

    match key.code {
        KeyCode::Enter => {
            let msg = std::mem::take(&mut chat.input);
            chat.push_message(chat.user_name.clone(), msg);
        }
        KeyCode::Backspace => {
            chat.input.pop();
        }
        KeyCode::Char(ch) => {
            // ignore CTRL combos (already handled above)
            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                chat.input.push(ch);
            }
        }

        // history scrolling
        KeyCode::Up => chat.scroll_up(),
        KeyCode::Down => chat.scroll_down(),
        KeyCode::PageUp => chat.page_up(),
        KeyCode::PageDown => chat.page_down(),

        _ => {}
    }

    false
}
