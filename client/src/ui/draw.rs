use crate::ui::{
    app::App,
    form::{draw, view::FormKind},
    view::{Chat, MenuPageKind, MenuState, View},
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use unicode_width::UnicodeWidthStr;

pub fn ui(f: &mut Frame, app: &App) {
    match &app.view {
        View::Menu(menu) => draw_menu(f, app, menu),
        View::Form(form) => match &form.kind {
            FormKind::Login(login) => draw::login_form(f, form, login),
            FormKind::Signup(signup) => draw::signup_form(f, form, signup),
        },
        View::Info(_) => draw_error(f, "Info view not implemented yet."),
        View::Chat(_) => draw_error(f, "Chat view not implemented yet."),
    }
}

fn draw_menu(f: &mut Frame, app: &App, menu: &MenuState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(f.size());

    let nav_hint = if app.session.is_some() {
        if menu.stack.len() > 1 {
            "Enter: select | Esc/Ctrl+C: back"
        } else {
            "Enter: select | Esc/Ctrl+C: logout"
        }
    } else {
        "Enter: select | Esc/Ctrl+C: quit"
    };

    // Header shows the navigation path + user + shortcuts
    let path = menu_path(menu);
    let mut header_spans = vec![
        Span::styled(path, Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("   "),
    ];

    if let Some(session) = app.session.as_ref() {
        header_spans.push(Span::raw("User: "));
        header_spans.push(Span::styled(
            session.username(),
            Style::default().add_modifier(Modifier::BOLD),
        ));
        header_spans.push(Span::raw("   "));
    }

    header_spans.push(Span::raw(nav_hint));

    let header = Line::from(header_spans);
    f.render_widget(Paragraph::new(Text::from(header)), chunks[0]);

    let entries = app.menu_entries(menu.current().kind);
    let items: Vec<ListItem> = entries
        .iter()
        .map(|entry| ListItem::new(entry.label.clone()))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(menu.current().kind.title())
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("▸ ");

    let mut state = ListState::default();
    if !entries.is_empty() {
        let selected = menu.current().selected.min(entries.len().saturating_sub(1));
        state.select(Some(selected));
    }

    f.render_stateful_widget(list, chunks[1], &mut state);

    let status_line = Line::from(vec![
        Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(app.menu_status()),
    ]);
    let status = Paragraph::new(Text::from(status_line));
    f.render_widget(status, chunks[2]);
}

fn draw_error(f: &mut Frame, message: &str) {
    let block = Block::default().title("Erreur").borders(Borders::ALL);
    let paragraph = Paragraph::new(message)
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, f.size());
}

fn menu_path(menu: &MenuState) -> String {
    let mut parts = Vec::new();
    for frame in &menu.stack {
        parts.push(frame.kind.title());
    }
    parts.join(" > ")
}

fn draw_chat(f: &mut Frame, chat: &Chat, authenticated: bool) {
    // Global layout: top (header) / middle (chat+users) / bottom (input)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(f.size());

    draw_header(f, chunks[0], chat, authenticated);
    draw_middle(f, chunks[1], chat);
    draw_input(f, chunks[2], chat);
}

fn draw_header(f: &mut Frame, area: Rect, chat: &Chat, authenticated: bool) {
    let hint = if authenticated {
        "Esc/Ctrl+C: logout | ↑↓ PgUp/PgDn: scroll"
    } else {
        "Esc/Ctrl+C: back | ↑↓ PgUp/PgDn: scroll"
    };

    let title = Line::from(vec![
        Span::styled(
            &chat.room_name,
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::raw("User: "),
        Span::styled(
            &chat.user_name,
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::raw(hint),
    ]);

    let header = Paragraph::new(Text::from(title));
    f.render_widget(header, area);
}

fn is_guest_root(menu: &MenuState) -> bool {
    menu.stack.len() == 1 && matches!(menu.current().kind, MenuPageKind::LoggedOut)
}

fn draw_middle(f: &mut Frame, area: Rect, chat: &Chat) {
    // Two columns: chat + users (optional)
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(area);

    draw_chat_history(f, cols[0], chat);
    draw_users(f, cols[1], chat);
}

fn draw_chat_history(f: &mut Frame, area: Rect, chat: &Chat) {
    let block = Block::default().title("History").borders(Borders::ALL);

    // Build chat lines
    let mut lines: Vec<Line> = Vec::new();
    for m in &chat.messages {
        let prefix = format!("[{}] {}: ", m.timestamp, m.author);
        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(m.content.clone()),
        ]));
    }

    // Paragraph scroll: (vertical, horizontal)
    let inner_height = area.height.saturating_sub(2); // borders
    let total_lines = lines.len() as u16;

    // if total_lines <= inner_height => offset = 0
    // else offset = total_lines - inner_height - scroll_from_bottom
    let max_offset = total_lines.saturating_sub(inner_height);
    let offset = max_offset.saturating_sub(chat.scroll_from_bottom);

    let paragraph = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((offset, 0));

    f.render_widget(paragraph, area);
}

fn draw_users(f: &mut Frame, area: Rect, chat: &Chat) {
    let block = Block::default().title("Users").borders(Borders::ALL);

    let lines: Vec<Line> = chat
        .users
        .iter()
        .map(|u| {
            if u == &chat.user_name {
                Line::from(Span::styled(
                    format!("• {}", u),
                    Style::default().add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(Span::raw(format!("• {}", u)))
            }
        })
        .collect();

    let paragraph = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn draw_input(f: &mut Frame, area: Rect, chat: &Chat) {
    let block = Block::default().title("Message").borders(Borders::ALL);

    // Render input text
    let paragraph = Paragraph::new(chat.input.clone())
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);

    // Input cursor (simple: end of string)
    // x = left border + 1 + unicode width
    let x = area.x + 1 + UnicodeWidthStr::width(chat.input.as_str()) as u16;
    let y = area.y + 1;
    f.set_cursor(x, y);
}
