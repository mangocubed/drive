use sdk::auth_client::UserInfo;

use crate::server::db_pool;
use crate::server::models::User;

use super::finish_all_sessions_by_user;

pub async fn disable_user(user: &User<'_>) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    if user.is_disabled() {
        return Ok(());
    }

    sqlx::query!(
        "UPDATE users SET disabled_at = current_timestamp WHERE disabled_at IS NULL AND id = $1",
        user.id
    )
    .execute(db_pool)
    .await?;

    finish_all_sessions_by_user(user).await?;

    Ok(())
}

pub async fn get_user_by_session_token<'a>(token: &str) -> sqlx::Result<User<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = (SELECT user_id FROM sessions WHERE token = $1 AND finished_at IS NULL AND mango3_auth_expires_at > current_timestamp LIMIT 1)
        LIMIT 1",
        token
    )
    .fetch_one(db_pool)
    .await
}

pub async fn insert_or_update_user<'a>(user_info: &UserInfo<'_>) -> sqlx::Result<User<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        User,
        "INSERT INTO users (
            mango3_user_id,
            username,
            email,
            display_name,
            initials,
            full_name,
            birthdate,
            language_code,
            country_alpha2
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (username) DO UPDATE SET
            mango3_user_id = $1,
            email = $3,
            display_name = $4,
            initials = $5,
            full_name = $6,
            birthdate = $7,
            language_code = $8,
            country_alpha2 = $9
        RETURNING *",
        user_info.id,              // $1
        &user_info.username,       // $2
        &user_info.email,          // $3
        &user_info.display_name,   // $4
        &user_info.initials,       // $5
        &user_info.full_name,      // $6
        user_info.birthdate,       // $7
        &user_info.language_code,  // $8
        &user_info.country_alpha2, // $9
    )
    .fetch_one(db_pool)
    .await
}
