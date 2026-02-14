pub const JWT_AUTH_TTL: i64 = 60; // 1 minute
pub const JWT_ACCESS_TTL: i64 = 60 * 15; // 15 minutes
pub const JWT_REFRESH_TTL: i64 = 60 * 60 * 24 * 7; // 7 days

pub const JWT_AUDIENCE: &str = "dydyscord-client";

pub const AUTH_HEADER: &str = "auth-token"; // Authentication cookie name

pub const SERVER_ADDR: &str = "0.0.0.0:3000";

pub const PG_MAX_CONNECTION: u32 = 5;

pub const REDIS_OPAQUE_STATE_TTL: u64 = 60 * 2; // 2 minutes
