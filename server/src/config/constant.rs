pub const JWT_AUDIENCE: &str = "dydyscord-client";

pub const JWT_AUTH_TTL: i64 = 60; // 1 minute
pub const JWT_SESSION_TTL: i64 = 60 * 60 * 2; // 2 hours

pub const JWT_AUTH_HEADER: &str = "auth-token";
pub const JWT_SESSION_HEADER: &str = "session-token";

pub const SERVER_ADDR: &str = "0.0.0.0:3000";

pub const PG_MAX_CONNECTION: u32 = 5;

pub const REDIS_OPAQUE_STATE_TTL: u64 = 60 * 2; // 2 minutes
