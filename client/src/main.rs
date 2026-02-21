mod auth;
mod config;
mod core;
mod error;
mod mls;
mod storage;
mod transport;
mod ui;

fn main() {
    let app = ui::app::App::new();

    if let Err(e) = ui::driver::run(app) {
        eprintln!("UI error: {e}");
    }
}
