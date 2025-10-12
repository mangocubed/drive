use sdk::auth_client::Auth;

use crate::server::config::USERS_CONFIG;
use crate::server::db_pool;
use crate::server::models::{Session, User};

use super::generate_random_string;

pub async fn finish_session(session: &Session<'_>) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!(
        "UPDATE sessions SET finished_at = current_timestamp WHERE id = $1",
        session.id
    )
    .execute(db_pool)
    .await
    .map(|_| ())
}

pub async fn finish_all_sessions_by_user(user: &User<'_>) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!(
        "UPDATE sessions SET finished_at = current_timestamp WHERE user_id = $1",
        user.id
    )
    .execute(db_pool)
    .await
    .map(|_| ())
}

pub async fn get_session_by_token<'a>(token: &str) -> sqlx::Result<Session<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Session,
        "SELECT * FROM sessions WHERE token = $1 AND finished_at IS NULL AND mango3_auth_expires_at > current_timestamp
        LIMIT 1",
        token
    )
    .fetch_one(db_pool)
    .await
}

pub async fn insert_session<'a>(user: &User<'_>, auth: &Auth<'_>) -> sqlx::Result<Session<'a>> {
    let db_pool = db_pool().await;

    let token = generate_random_string(USERS_CONFIG.session_token_length);

    sqlx::query_as!(
        Session,
        "INSERT INTO sessions (
            user_id, token, mango3_auth_token, mango3_auth_expires_at, mango3_auth_refreshed_at
        ) VALUES ($1, $2, $3, $4, $5)
        RETURNING *",
        user.id,           // $1
        token,             // $2
        &auth.token,       // $3
        auth.expires_at,   // $4
        auth.refreshed_at  // $5
    )
    .fetch_one(db_pool)
    .await
}
