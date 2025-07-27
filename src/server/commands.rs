use std::borrow::Cow;

use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHasher};
use validator::{Validate, ValidationError, ValidationErrors};

use crate::inputs::RegisterInput;

use super::constants::ERROR_ALREADY_EXISTS;
use super::db_pool;
use super::models::User;

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

fn encrypt_password(value: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2.hash_password(value.as_bytes(), &salt).unwrap().to_string()
}

pub async fn insert_user<'a>(input: RegisterInput) -> Result<User<'a>, ValidationErrors> {
    input.validate()?;

    let mut validation_errors = ValidationErrors::new();

    if username_exists(&input.username).await {
        validation_errors.add(
            "username",
            ValidationError::new(ERROR_ALREADY_EXISTS).with_message(Cow::Borrowed("Already exists")),
        );
    }

    if email_exists(&input.email).await {
        validation_errors.add(
            "email",
            ValidationError::new(ERROR_ALREADY_EXISTS).with_message(Cow::Borrowed("Already exists")),
        );
    }

    if !validation_errors.is_empty() {
        return Err(validation_errors);
    }

    let db_pool = db_pool().await;
    let display_name = input.full_name.split(' ').next().unwrap();

    sqlx::query_as!(
        User,
        "INSERT INTO users (
            username,
            email,
            encrypted_password,
            display_name,
            full_name,
            birthdate,
            language_code
        ) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
        input.username,                    // $1
        input.email.to_lowercase(),        // $2
        encrypt_password(&input.password), // $3
        display_name,                      // $4
        input.full_name,                   // $5
        input.birthdate,                   // $6
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
    use super::*;

    #[tokio::test]
    async fn should_find_an_email() {}

    #[tokio::test]
    async fn should_not_find_an_email() {}

    #[tokio::test]
    async fn should_find_an_username() {}

    #[tokio::test]
    async fn should_not_find_an_username() {}

    #[tokio::test]
    async fn should_insert_a_new_user() {}

    #[tokio::test]
    async fn should_fail_to_insert_a_new_user() {}
}
