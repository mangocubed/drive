use crate::server::config::USERS_CONFIG;
use crate::server::db_pool;
use crate::server::models::{AccessToken, User};

use super::generate_random_string;

pub async fn delete_access_token(access_token: &AccessToken<'_>) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!("DELETE FROM access_tokens WHERE id = $1", access_token.id)
        .execute(db_pool)
        .await
        .map(|_| ())
}

pub async fn delete_all_access_tokens_by_user(user: &User<'_>) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!("DELETE FROM access_tokens WHERE user_id = $1", user.id)
        .execute(db_pool)
        .await
        .map(|_| ())
}

pub async fn get_access_token<'a>(token: &str) -> sqlx::Result<AccessToken<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        AccessToken,
        "SELECT * FROM access_tokens WHERE token = $1 LIMIT 1",
        token
    )
    .fetch_one(db_pool)
    .await
}

pub async fn insert_access_token<'a>(user: &User<'_>) -> sqlx::Result<AccessToken<'a>> {
    let db_pool = db_pool().await;

    let token = generate_random_string(USERS_CONFIG.access_token_length);

    sqlx::query_as!(
        AccessToken,
        "INSERT INTO access_tokens (user_id, token) VALUES ($1, $2) RETURNING *",
        user.id, // $1
        token    // $2
    )
    .fetch_one(db_pool)
    .await
}
