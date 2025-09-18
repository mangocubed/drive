use chrono::NaiveDate;
use validator::{Validate, ValidationErrors};

use crate::inputs::RegisterInput;
use crate::server::config::USERS_CONFIG;
use crate::server::constants::ERROR_ALREADY_EXISTS;
use crate::server::db_pool;
use crate::server::models::User;

use super::{delete_all_access_tokens_by_user, encrypt_password};

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

    delete_all_access_tokens_by_user(user).await?;

    Ok(())
}

async fn email_exists(value: &str) -> bool {
    let db_pool = db_pool().await;

    sqlx::query!(
        "SELECT id FROM users WHERE LOWER(email) = $1 LIMIT 1",
        value.to_lowercase() // $1
    )
    .fetch_one(db_pool)
    .await
    .is_ok()
}

pub async fn get_user_by_access_token<'a>(token: &str) -> sqlx::Result<User<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = (SELECT user_id FROM access_tokens WHERE token = $1 LIMIT 1) LIMIT 1",
        token
    )
    .fetch_one(db_pool)
    .await
}

async fn get_users_count() -> sqlx::Result<i64> {
    let db_pool = db_pool().await;

    sqlx::query!(r#"SELECT COUNT(*) as "count!" FROM users WHERE disabled_at IS NOT NULL"#)
        .fetch_one(db_pool)
        .await
        .map(|row| row.count)
}

pub async fn insert_user<'a>(input: &RegisterInput) -> Result<User<'a>, ValidationErrors> {
    if get_users_count().await.unwrap_or_default() >= USERS_CONFIG.limit.into() {
        return Err(ValidationErrors::new());
    }

    input.validate()?;

    let mut validation_errors = ValidationErrors::new();

    if username_exists(&input.username).await {
        validation_errors.add("username", ERROR_ALREADY_EXISTS.clone());
    }

    if email_exists(&input.email).await {
        validation_errors.add("email", ERROR_ALREADY_EXISTS.clone());
    }

    if !validation_errors.is_empty() {
        return Err(validation_errors);
    }

    let db_pool = db_pool().await;
    let display_name = input.full_name.split(' ').next().unwrap();
    let birthdate = NaiveDate::parse_from_str(&input.birthdate, "%Y-%m-%d").unwrap();

    sqlx::query_as!(
        User,
        "INSERT INTO users (
            username,
            email,
            encrypted_password,
            display_name,
            full_name,
            birthdate,
            country_alpha2
        ) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
        input.username,                    // $1
        input.email.to_lowercase(),        // $2
        encrypt_password(&input.password), // $3
        display_name,                      // $4
        input.full_name,                   // $5
        birthdate,                         // $6
        input.country_alpha2,              // $7
    )
    .fetch_one(db_pool)
    .await
    .map_err(|_| ValidationErrors::new())
}

async fn username_exists(value: &str) -> bool {
    let db_pool = db_pool().await;

    sqlx::query!(
        "SELECT id FROM users WHERE LOWER(username) = $1 LIMIT 1",
        value.to_lowercase()
    )
    .fetch_one(db_pool)
    .await
    .is_ok()
}

#[cfg(test)]
mod tests {
    use crate::test_utils::*;

    use super::*;

    #[tokio::test]
    async fn should_find_existing_email() {
        let user = insert_test_user(None).await;

        assert!(email_exists(&user.email).await);
    }

    #[tokio::test]
    async fn should_not_find_inexistent_email() {
        let email = fake_email();
        assert!(!email_exists(&email).await);
    }

    #[tokio::test]
    async fn should_find_an_existing_username() {
        let user = insert_test_user(None).await;

        assert!(username_exists(&user.username).await);
    }

    #[tokio::test]
    async fn should_not_find_inexistent_username() {
        let username = fake_username();

        assert!(!username_exists(&username).await);
    }

    #[tokio::test]
    async fn should_insert_a_user() {
        let input = RegisterInput {
            username: fake_username(),
            email: fake_email(),
            password: fake_password(),
            full_name: fake_name(),
            birthdate: fake_birthdate(),
            country_alpha2: fake_country_alpha2(),
        };

        let result = insert_user(&input).await;

        assert!(result.is_ok());

        let user = result.unwrap();

        assert_eq!(user.username, input.username);
        assert_eq!(user.email, input.email);
        assert_eq!(user.full_name, input.full_name);
        assert_eq!(user.birthdate.to_string(), input.birthdate);
        assert_eq!(user.country_alpha2, input.country_alpha2);
    }

    #[tokio::test]
    async fn should_not_insert_a_user_with_existent_username() {
        let user = insert_test_user(None).await;
        let input = RegisterInput {
            username: user.username.to_string(),
            email: fake_email(),
            password: fake_password(),
            full_name: fake_name(),
            birthdate: fake_birthdate(),
            country_alpha2: fake_country_alpha2(),
        };

        let result = insert_user(&input).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_not_insert_a_user_with_existent_email() {
        let user = insert_test_user(None).await;
        let input = RegisterInput {
            username: fake_username(),
            email: user.email.to_string(),
            password: fake_password(),
            full_name: fake_name(),
            birthdate: fake_birthdate(),
            country_alpha2: fake_country_alpha2(),
        };

        let result = insert_user(&input).await;

        assert!(result.is_err());
    }
}
