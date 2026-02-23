use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use unicode_width::UnicodeWidthStr;

use crate::ui::form::view::{
    CreateGroupFormStep, FormState, GroupCreateFormState, LoginField, LoginFormState, SignupField,
    SignupFormState,
};

// ========================================
// Form: Log In
// ========================================

pub fn login_form(f: &mut Frame, form: &FormState, state: &LoginFormState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(3)])
        .split(f.size());

    let header = Line::from(vec![
        Span::styled("Log In", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("   "),
        Span::raw("Enter: next/submit | Esc/Ctrl+C: back"),
    ]);
    f.render_widget(Paragraph::new(Text::from(header)), chunks[0]);

    let username_label = "Username: ";
    let password_label = "Password: ";
    let password_mask = "*".repeat(state.password_len());

    let username_style = if state.active == LoginField::Username {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let password_style = if state.active == LoginField::Password {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(username_label, username_style),
        Span::raw(state.username.clone()),
    ]));
    lines.push(Line::from(vec![
        Span::styled(password_label, password_style),
        Span::raw(password_mask.as_str()),
    ]));

    if let Some(error) = &form.error {
        lines.push(Line::from(vec![
            Span::styled("Error: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(error.clone()),
        ]));
    }

    let block = Block::default().title("Credentials").borders(Borders::ALL);
    let paragraph = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunks[1]);

    let (label, value_width, row) = match state.active {
        LoginField::Username => (username_label, state.username.as_str(), 0u16),
        LoginField::Password => (password_label, password_mask.as_str(), 1u16),
    };
    let x = chunks[1].x
        + 1
        + UnicodeWidthStr::width(label) as u16
        + UnicodeWidthStr::width(value_width) as u16;
    let y = chunks[1].y + 1 + row;
    f.set_cursor(x, y);
}

// ========================================
// Form: Sign up
// ========================================

pub fn signup_form(f: &mut Frame, form: &FormState, state: &SignupFormState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(3)])
        .split(f.size());

    let header = Line::from(vec![
        Span::styled("Sign up", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("   "),
        Span::raw("Enter: next/submit | Esc/Ctrl+C: back"),
    ]);
    f.render_widget(Paragraph::new(Text::from(header)), chunks[0]);

    let username_label = "Username: ";
    let password_label = "Password: ";
    let confirm_password_label = "Confirm Password: ";
    let password_mask = "*".repeat(state.password_len());
    let confirm_password_mask = "*".repeat(state.confirm_len());

    let username_style = if state.active == SignupField::Username {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let password_style = if state.active == SignupField::Password {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let confirm_password_style = if state.active == SignupField::ConfirmPassword {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(username_label, username_style),
        Span::raw(state.username.clone()),
    ]));
    lines.push(Line::from(vec![
        Span::styled(password_label, password_style),
        Span::raw(password_mask.as_str()),
    ]));
    lines.push(Line::from(vec![
        Span::styled(confirm_password_label, confirm_password_style),
        Span::raw(confirm_password_mask.as_str()),
    ]));
    if let Some(error) = &form.error {
        lines.push(Line::from(vec![
            Span::styled("Error: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(error.clone()),
        ]));
    }

    let block = Block::default().title("Credentials").borders(Borders::ALL);
    let paragraph = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunks[1]);

    let (label, value_width, row) = match state.active {
        SignupField::Username => (username_label, state.username.as_str(), 0u16),
        SignupField::Password => (password_label, password_mask.as_str(), 1u16),
        SignupField::ConfirmPassword => {
            (confirm_password_label, confirm_password_mask.as_str(), 2u16)
        }
    };
    let x = chunks[1].x
        + 1
        + UnicodeWidthStr::width(label) as u16
        + UnicodeWidthStr::width(value_width) as u16;
    let y = chunks[1].y + 1 + row;
    f.set_cursor(x, y);
}

// ========================================
// Form: Group create
// ========================================

pub fn group_create_form(f: &mut Frame, form: &FormState, state: &GroupCreateFormState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(3)])
        .split(f.size());

    let header = Line::from(vec![
        Span::styled(
            "Create group",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::raw("Enter: next/submit | Esc/Ctrl+C: back"),
    ]);
    f.render_widget(Paragraph::new(Text::from(header)), chunks[0]);

    match state.step {
        CreateGroupFormStep::Info => {
            let groupname_label = "Group name: ";
            let groupname_style = Style::default().add_modifier(Modifier::BOLD);

            let mut lines = Vec::new();
            lines.push(Line::from(vec![
                Span::styled(groupname_label, groupname_style),
                Span::raw(state.name.clone()),
            ]));
            if let Some(error) = &form.error {
                lines.push(Line::from(vec![
                    Span::styled("Error: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(error.clone()),
                ]));
            }

            let block = Block::default().title("Info").borders(Borders::ALL);
            let paragraph = Paragraph::new(Text::from(lines))
                .block(block)
                .wrap(Wrap { trim: true });
            f.render_widget(paragraph, chunks[1]);

            let x = chunks[1].x
                + 1
                + UnicodeWidthStr::width(groupname_label) as u16
                + UnicodeWidthStr::width(state.name.as_str()) as u16;
            let y = chunks[1].y + 1;
            f.set_cursor(x, y);
        }
        CreateGroupFormStep::Members => {
            let items: Vec<ListItem> = state
                .friends
                .iter()
                .map(|friend| {
                    let mark = if friend.selected { "[x]" } else { "[ ]" };
                    ListItem::new(format!("{} {}", mark, friend.username))
                })
                .collect();

            let mut list_state = ListState::default();
            if !items.is_empty() {
                let selected = state.cursor.min(items.len().saturating_sub(1));
                list_state.select(Some(selected));
            }

            let list = List::new(items)
                .block(Block::default().title("Invite friends").borders(Borders::ALL))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol("▸ ");

            f.render_stateful_widget(list, chunks[1], &mut list_state);
        }
    }
}
