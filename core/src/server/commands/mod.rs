use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use bytesize::ByteSize;
use rand::distr::Alphanumeric;
use rand::{Rng, rng};
use strum::IntoEnumIterator;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

use crate::enums::FileVisibility;
use crate::inputs::{FolderInput, LoginInput};
use crate::server::config::{APP_CONFIG, STORAGE_CONFIG};

use super::constants::*;
use super::db_pool;
use super::models::{File, Folder, User};

mod access_token_commands;
mod file_commands;
mod folder_commands;
mod folder_item_commands;
mod plan_commands;
mod trash_commands;
mod user_commands;

pub use access_token_commands::*;
pub use file_commands::*;
pub use folder_commands::*;
pub use folder_item_commands::*;
pub use plan_commands::*;
pub use trash_commands::*;
pub use user_commands::*;

pub async fn authenticate_user<'a>(input: &LoginInput) -> Result<User<'a>, ValidationErrors> {
    input.validate()?;

    let db_pool = db_pool().await;

    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE disabled_at IS NULL AND (LOWER(username) = $1 OR LOWER(email) = $1)
        LIMIT 1",
        input.username_or_email.to_lowercase()
    )
    .fetch_one(db_pool)
    .await
    .map_err(|_| ValidationErrors::new())?;

    if user.verify_password(&input.password) {
        Ok(user)
    } else {
        Err(ValidationErrors::new())
    }
}

fn encrypt_password(value: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2.hash_password(value.as_bytes(), &salt).unwrap().to_string()
}

pub async fn enable_user(user: &User<'_>) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    if !user.is_disabled() {
        return Ok(());
    }

    sqlx::query!(
        "UPDATE users SET disabled_at = NULL WHERE disabled_at IS NOT NULL AND id = $1",
        user.id
    )
    .execute(db_pool)
    .await
    .map(|_| ())
}

async fn file_name_exists(user: &User<'_>, parent_folder_id: Option<Uuid>, name: &str) -> bool {
    let db_pool = db_pool().await;

    sqlx::query!(
        "(
            SELECT id FROM files
            WHERE user_id = $1
                AND (($2::uuid IS NULL AND parent_folder_id IS NULL) OR parent_folder_id = $2) AND LOWER(name) = $3
            LIMIT 1
        ) UNION (
            SELECT id FROM folders
            WHERE user_id = $1
                AND (($2::uuid IS NULL AND parent_folder_id IS NULL) OR parent_folder_id = $2) AND LOWER(name) = $3
            LIMIT 1
        )",
        user.id,             // $1
        parent_folder_id,    // $2
        name.to_lowercase()  // $3
    )
    .fetch_one(db_pool)
    .await
    .is_ok()
}

fn generate_random_string(length: u8) -> String {
    rng()
        .sample_iter(&Alphanumeric)
        .take(length as usize)
        .map(char::from)
        .collect()
}

pub fn get_available_space() -> ByteSize {
    let stats = uucore::fsext::statfs(STORAGE_CONFIG.path().as_os_str()).expect("Could not get storage stats");

    ByteSize(stats.f_bavail * stats.f_bsize as u64)
}

pub async fn get_file_by_id<'a>(id: Uuid, user: Option<&User<'_>>) -> sqlx::Result<File<'a>> {
    let db_pool = db_pool().await;
    let user_id = user.map(|u| u.id);

    sqlx::query_as!(
        File,
        r#"SELECT
            id,
            user_id,
            parent_folder_id,
            name,
            visibility as "visibility!: FileVisibility",
            media_type,
            byte_size,
            md5_checksum,
            trashed_at,
            created_at,
            updated_at
        FROM files WHERE id = $1 AND ($2::uuid IS NULL OR user_id = $2) LIMIT 1"#,
        id,      // $1
        user_id, // $2
    )
    .fetch_one(db_pool)
    .await
}

pub async fn get_folder_by_id<'a>(id: Uuid, user: Option<&User<'_>>) -> sqlx::Result<Folder<'a>> {
    let db_pool = db_pool().await;
    let user_id = user.map(|u| u.id);

    sqlx::query_as!(
        Folder,
        r#"SELECT
            id,
            user_id,
            parent_folder_id,
            name,
            visibility as "visibility!: FileVisibility",
            trashed_at,
            created_at,
            updated_at
        FROM folders WHERE id = $1 AND ($2::uuid IS NULL OR user_id = $2) LIMIT 1"#,
        id,      // $1
        user_id, // $2
    )
    .fetch_one(db_pool)
    .await
}

pub async fn get_used_space_by_user(user: &User<'_>) -> ByteSize {
    let db_pool = db_pool().await;

    sqlx::query!(
        r#"SELECT SUM(byte_size)::bigint AS "used_space!" FROM files WHERE user_id = $1"#,
        user.id
    )
    .fetch_one(db_pool)
    .await
    .map(|row| ByteSize(row.used_space as u64))
    .unwrap_or_default()
}

pub async fn get_user_by_id<'a>(id: Uuid) -> sqlx::Result<User<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1 LIMIT 1", id)
        .fetch_one(db_pool)
        .await
}

pub async fn get_user_by_username(username: &str) -> sqlx::Result<User<'_>> {
    if username.is_empty() {
        return Err(sqlx::Error::RowNotFound);
    }

    let db_pool = db_pool().await;

    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE LOWER(username) = $1 LIMIT 1",
        username.to_lowercase()
    )
    .fetch_one(db_pool)
    .await
}

pub async fn insert_folder<'a>(user: &User<'_>, input: &FolderInput) -> Result<Folder<'a>, ValidationErrors> {
    input.validate()?;

    let mut validation_errors = ValidationErrors::new();
    let db_pool = db_pool().await;

    if file_name_exists(user, input.parent_folder_id, &input.name).await {
        validation_errors.add("name", ERROR_ALREADY_EXISTS.clone());
    }

    if let Some(parent_folder_id) = input.parent_folder_id {
        if let Ok(parent_folder) = get_folder_by_id(parent_folder_id, Some(user)).await {
            if !FileVisibility::iter()
                .skip_while(|value| *value != parent_folder.visibility)
                .any(|value| value == input.visibility)
            {
                validation_errors.add("visibility", ERROR_IS_INVALID.clone());
            }
        } else {
            validation_errors.add("parent_folder_id", ERROR_IS_INVALID.clone());
        }
    }

    if !validation_errors.is_empty() {
        return Err(validation_errors);
    }

    sqlx::query_as!(
        Folder,
        r#"INSERT INTO folders (user_id, parent_folder_id, name, visibility) VALUES ($1, $2, $3, $4)
        RETURNING
            id,
            user_id,
            parent_folder_id,
            name,
            visibility as "visibility!: FileVisibility",
            trashed_at,
            created_at,
            updated_at"#,
        user.id,                // $1
        input.parent_folder_id, // $2
        input.name,             // $3
        input.visibility as _   // $4
    )
    .fetch_one(db_pool)
    .await
    .map_err(|_| ValidationErrors::new())
}

pub fn verify_app_token(app_token: &str) -> bool {
    app_token == APP_CONFIG.token || APP_CONFIG.old_tokens.contains(&app_token.to_owned())
}

pub(crate) fn verify_password(encrypted_password: &str, password: &str) -> bool {
    let argon2 = Argon2::default();

    let Ok(password_hash) = PasswordHash::new(encrypted_password) else {
        return false;
    };

    argon2.verify_password(password.as_bytes(), &password_hash).is_ok()
}

#[cfg(test)]
mod tests {
    use crate::test_utils::*;

    use super::*;

    #[tokio::test]
    async fn should_authenticate_user() {
        let password = fake_password();
        let user = insert_test_user(Some(&password)).await;
        let input = LoginInput {
            username_or_email: user.username.to_string(),
            password: password.clone(),
        };

        let result = authenticate_user(&input).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_not_authenticate_user_with_invalid_input() {
        let input = LoginInput {
            username_or_email: fake_username(),
            password: fake_password(),
        };

        let result = authenticate_user(&input).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_get_seven_folders_by_user() {
        let user = insert_test_user(None).await;

        insert_test_folders(7, Some(&user), None).await;

        let result = get_all_folder_items(Some(&user), None).await;

        assert!(result.is_ok());

        let folders = result.unwrap();

        assert_eq!(folders.len(), 7);
    }

    #[tokio::test]
    async fn should_get_seven_folders_by_user_with_parent_folder() {
        let user = insert_test_user(None).await;
        let parent_folder = insert_test_folder(Some(&user), None).await;

        insert_test_folders(7, Some(&user), Some(&parent_folder)).await;

        let result = get_all_folder_items(Some(&user), Some(&parent_folder)).await;

        assert!(result.is_ok());

        let folders = result.unwrap();

        assert_eq!(folders.len(), 7);
    }

    #[tokio::test]
    async fn should_get_zero_folders_by_user() {
        let user = insert_test_user(None).await;

        let result = get_all_folder_items(Some(&user), None).await;

        assert!(result.is_ok());

        let folders = result.unwrap();

        assert_eq!(folders.len(), 0);
    }

    #[tokio::test]
    async fn should_get_folder_by_id() {
        let user = insert_test_user(None).await;
        let folder = insert_test_folder(Some(&user), None).await;

        let result = get_folder_by_id(folder.id, Some(&user)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_not_get_folder_by_id_with_invalid_user() {
        let invalid_user = insert_test_user(None).await;
        let folder = insert_test_folder(None, None).await;

        let result = get_folder_by_id(folder.id, Some(&invalid_user)).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_insert_a_folder() {
        let user = insert_test_user(None).await;
        let input = FolderInput {
            parent_folder_id: None,
            name: fake_name(),
            visibility: FileVisibility::Private,
        };

        let result = insert_folder(&user, &input).await;

        assert!(result.is_ok());

        let folder = result.unwrap();

        assert_eq!(folder.user_id, user.id);
        assert!(folder.parent_folder_id.is_none());
        assert_eq!(folder.name, input.name);
    }

    #[tokio::test]
    async fn should_insert_a_folder_with_parent_folder() {
        let user = insert_test_user(None).await;
        let parent_folder = insert_test_folder(Some(&user), None).await;
        let input = FolderInput {
            parent_folder_id: Some(parent_folder.id),
            name: fake_name(),
            visibility: FileVisibility::Private,
        };

        let result = insert_folder(&user, &input).await;

        assert!(result.is_ok());

        let folder = result.unwrap();

        assert_eq!(folder.user_id, user.id);
        assert_eq!(folder.parent_folder_id, input.parent_folder_id);
        assert_eq!(folder.name, input.name);
    }

    #[tokio::test]
    async fn should_not_insert_a_folder_with_existent_name() {
        let user = insert_test_user(None).await;
        let folder = insert_test_folder(Some(&user), None).await;
        let input = FolderInput {
            parent_folder_id: None,
            name: folder.name.to_string(),
            visibility: FileVisibility::Private,
        };

        let result = insert_folder(&user, &input).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_not_insert_a_folder_with_invalid_parent_folder() {
        let invalid_parent_folder = insert_test_folder(None, None).await;
        let user = insert_test_user(None).await;
        let input = FolderInput {
            parent_folder_id: Some(invalid_parent_folder.id),
            name: fake_name(),
            visibility: FileVisibility::Private,
        };

        let result = insert_folder(&user, &input).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_get_used_storage_equal_zero_when_is_empty() {
        let user = insert_test_user(None).await;

        let used_storage = get_used_space_by_user(&user).await;

        assert_eq!(used_storage, ByteSize(0));
    }

    #[tokio::test]
    async fn should_get_used_storage_more_than_zero_after_insert() {
        let user = insert_test_user(None).await;

        insert_test_files(7, Some(&user)).await;

        let used_storage = get_used_space_by_user(&user).await;

        assert!(used_storage > ByteSize(0));
    }
}
