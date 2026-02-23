use crate::ui::{
    app::App,
    chat::draw as chat_draw,
    form::{draw as form_draw, view::FormKind},
    info::draw as info_draw,
    menu::draw as menu_draw,
    view::View,
};
use ratatui::Frame;

pub fn ui(f: &mut Frame, app: &App) {
    match &app.view {
        View::Menu(menu) => menu_draw::menu(f, app, menu),
        View::Form(form) => match &form.kind {
            FormKind::Login(login) => form_draw::login_form(f, form, login),
            FormKind::Signup(signup) => form_draw::signup_form(f, form, signup),
        },
        View::Info(info) => info_draw::info(f, info),
        View::Chat(chat) => chat_draw::chat(f, chat, app.session.is_some()),
    }
}
