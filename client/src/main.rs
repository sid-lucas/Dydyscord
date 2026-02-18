mod app;
mod auth;
mod config;
mod error;
mod mls;
mod storage;
mod transport;
mod ui;

fn main() {
    let mut state = app::state::AppState::new();
    app::nav::run(&mut state);
}
