mod api;
mod app;
mod choice;
mod constants;
mod error;
mod mls;
mod opaque;
mod session;

fn main() {
    if let Err(e) = app::run() {
        eprintln!("Error: {e}");
    }
}
