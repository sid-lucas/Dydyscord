use crate::ui::{
    app::App,
    menu::view::{MenuPageKind, MenuState},
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

pub fn menu(f: &mut Frame, app: &App, menu: &MenuState) {
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

fn menu_path(menu: &MenuState) -> String {
    let mut parts = Vec::new();
    for frame in &menu.stack {
        parts.push(frame.kind.title());
    }
    parts.join(" > ")
}

fn is_guest_root(menu: &MenuState) -> bool {
    menu.stack.len() == 1 && matches!(menu.current().kind, MenuPageKind::LoggedOut)
}
