use ratatui::{
    Frame,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::ui::info::view::InfoState;

pub fn info(f: &mut Frame, _info: &InfoState) {
    let block = Block::default().title("Info").borders(Borders::ALL);
    let paragraph = Paragraph::new("Info view not implemented yet.")
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, f.size());
}
