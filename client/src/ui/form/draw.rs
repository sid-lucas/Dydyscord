use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use unicode_width::UnicodeWidthStr;

use crate::ui::form::view::{FormState, LoginField, LoginFormState, SignupField, SignupFormState};

pub fn login_form(f: &mut Frame, form: &FormState, login: &LoginFormState) {
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
    let password_mask = "*".repeat(login.password_len());

    let username_style = if login.active == LoginField::Username {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let password_style = if login.active == LoginField::Password {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(username_label, username_style),
        Span::raw(login.username.clone()),
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

    let (label, value_width, row) = match login.active {
        LoginField::Username => (username_label, login.username.as_str(), 0u16),
        LoginField::Password => (password_label, password_mask.as_str(), 1u16),
    };
    let x = chunks[1].x
        + 1
        + UnicodeWidthStr::width(label) as u16
        + UnicodeWidthStr::width(value_width) as u16;
    let y = chunks[1].y + 1 + row;
    f.set_cursor(x, y);
}

pub fn signup_form(f: &mut Frame, form: &FormState, signup: &SignupFormState) {
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
    let password_mask = "*".repeat(signup.password_len());
    let confirm_password_mask = "*".repeat(signup.confirm_len());

    let username_style = if signup.active == SignupField::Username {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let password_style = if signup.active == SignupField::Password {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let confirm_password_style = if signup.active == SignupField::ConfirmPassword {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(username_label, username_style),
        Span::raw(signup.username.clone()),
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

    let (label, value_width, row) = match signup.active {
        SignupField::Username => (username_label, signup.username.as_str(), 0u16),
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
