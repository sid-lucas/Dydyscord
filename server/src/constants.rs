use std::cell::OnceCell;

pub const JWT_TTL: i64 = 60 * 60 * 60 * 24; // 1 day
pub const JWT_AUDIENCE: &str = "dydyscord-client"; // TODO change
pub const JWT_SECRET_KEY: OnceCell<String> = OnceCell::new(); // TODO change

pub const AUTH_HEADER: &str = "auth-token";

pub const SERVER_ADDR: &str = "0.0.0.0:3000";

// TODO : Mettre var environnement ici aussi?
