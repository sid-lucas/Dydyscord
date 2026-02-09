use std::cell::OnceCell;
use std::sync::OnceLock;

// TODO :peut etre utiliser OnceCell sur ces variables... a discuter

pub const JWT_TTL: i64 = 60 * 60 * 60 * 24; // 1 day
pub const JWT_AUDIENCE: &str = "dydyscord-client"; // TODO change
pub static JWT_SECRET_KEY: OnceLock<String> = OnceLock::new(); // TODO Change ?

pub const AUTH_HEADER: &str = "auth-token"; // Nom cookie d'authentification

pub const SERVER_ADDR: &str = "0.0.0.0:3000";

// TODO : Mettre var environnement ici aussi?
