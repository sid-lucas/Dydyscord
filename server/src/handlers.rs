use axum::{Json, extract::State};
use sqlx::PgPool;

use crate::models::{CreateUserPayload, User};

pub async fn root(State(pool): State<PgPool>) -> String {
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT id, username
        FROM users
        ORDER BY id
        LIMIT 100
        "#
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch users");

    users
        .into_iter()
        .map(|u| u.username)
        .collect::<Vec<_>>()
        .join(", ")
}

pub async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUserPayload>,
) -> Json<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (username)
        VALUES ($1)
        RETURNING id, username
        "#,
        payload.username
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to create user");

    Json(user)
}
