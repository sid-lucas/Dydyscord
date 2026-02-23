use crate::ui::{
    app::App,
    form::{draw as form_draw, view::FormKind},
    menu::draw as menu_draw,
    view::{Chat, View},
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use unicode_width::UnicodeWidthStr;

pub fn ui(f: &mut Frame, app: &App) {
    match &app.view {
        View::Menu(menu) => menu_draw::menu(f, app, menu),
        View::Form(form) => match &form.kind {
            FormKind::Login(login) => form_draw::login_form(f, form, login),
            FormKind::Signup(signup) => form_draw::signup_form(f, form, signup),
        },
        View::Info(_) => draw_error(f, "Info view not implemented yet."),
        View::Chat(_) => draw_error(f, "Chat view not implemented yet."),
    }
}

fn draw_error(f: &mut Frame, message: &str) {
    let block = Block::default().title("Erreur").borders(Borders::ALL);
    let paragraph = Paragraph::new(message)
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, f.size());
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
