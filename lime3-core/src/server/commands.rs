use std::fs::File as FsFile;
use std::io::Write;

use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHasher};
use bytesize::ByteSize;
use chrono::NaiveDate;
use file_format::FileFormat;
use md5::{Digest, Md5};
use polar_rs::{CheckoutSession, CheckoutSessionParams, PolarError, PolarResult};
use strum::IntoEnumIterator;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

use crate::enums::FileVisibility;
use crate::inputs::{FileInput, FolderInput, LoginInput, RegisterInput};
use crate::server::config::{BILLING_CONFIG, MEMBERSHIPS_CONFIG, MembershipConfig};
use crate::server::constants::ERROR_IS_TOO_LARGE;
use crate::server::models::FolderItem;

use super::constants::{ALLOWED_FILE_FORMATS, ERROR_ALREADY_EXISTS, ERROR_IS_INVALID};
use super::models::{File, Folder, User, UserSession};
use super::{POLAR_CLIENT, db_pool};

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

pub async fn create_checkout(
    user: &User<'_>,
    membership: &MembershipConfig,
    annual_billing: bool,
) -> PolarResult<CheckoutSession> {
    let product_id = if annual_billing {
        membership.annual.as_ref().map(|annual| annual.polar_product_id)
    } else {
        membership.monthly.as_ref().map(|monthly| monthly.polar_product_id)
    };

    let Some(product_id) = product_id else {
        return Err(PolarError::Request("Invalid membership".to_owned()));
    };

    let params = CheckoutSessionParams {
        products: vec![product_id],
        external_customer_id: Some(user.id.to_string()),
        customer_email: Some(user.email.to_string()),
        customer_name: Some(user.full_name.to_string()),
        success_url: Some(
            BILLING_CONFIG
                .success_base_url
                .join("checkout-success?checkout_id={CHECKOUT_ID}")
                .unwrap(),
        ),
        ..Default::default()
    };

    POLAR_CLIENT.create_checkout_session(&params).await
}

pub async fn delete_all_user_sessions_by_user(user: &User<'_>) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!("DELETE FROM user_sessions WHERE user_id = $1", user.id)
        .execute(db_pool)
        .await
        .map(|_| ())
}

pub async fn delete_user_session(user_session: &UserSession) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!("DELETE FROM user_sessions WHERE id = $1", user_session.id)
        .execute(db_pool)
        .await
        .map(|_| ())
}

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

    delete_all_user_sessions_by_user(user).await?;

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

fn encrypt_password(value: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2.hash_password(value.as_bytes(), &salt).unwrap().to_string()
}

async fn file_name_exists(user: &User<'_>, parent_folder_id: Option<Uuid>, name: &str) -> bool {
    let db_pool = db_pool().await;

    sqlx::query!(
        "(
            SELECT id FROM files
            WHERE user_id = $1 AND (($2::uuid IS NULL AND parent_folder_id IS NULL) OR parent_folder_id = $2)
                AND LOWER(name) = $3 LIMIT 1
        ) UNION (
            SELECT id FROM folders
            WHERE user_id = $1 AND (($2::uuid IS NULL AND parent_folder_id IS NULL) OR parent_folder_id = $2)
                AND LOWER(name) = $3 LIMIT 1
        )",
        user.id,             // $1
        parent_folder_id,    // $2
        name.to_lowercase()  // $3
    )
    .fetch_one(db_pool)
    .await
    .is_ok()
}

pub fn get_available_memberships_by_user(user: &User<'_>) -> Vec<MembershipConfig> {
    MEMBERSHIPS_CONFIG
        .options
        .iter()
        .filter(|option| !option.is_restricted && option.code != user.membership_code)
        .cloned()
        .collect()
}

pub fn get_membership_by_code(code: &str) -> Option<&MembershipConfig> {
    MEMBERSHIPS_CONFIG.options.iter().find(|option| option.code == code)
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
            created_at,
            updated_at
        FROM files WHERE id = $1 AND ($2::uuid IS NULL OR user_id = $2) LIMIT 1"#,
        id,      // $1
        user_id, // $2
    )
    .fetch_one(db_pool)
    .await
}

pub async fn get_all_folder_items_by_user<'a>(
    user: &User<'_>,
    parent_folder: Option<&Folder<'_>>,
) -> Vec<FolderItem<'a>> {
    let db_pool = db_pool().await;
    let parent_folder_id = parent_folder.map(|f| f.id);

    sqlx::query_as!(
        FolderItem,
        r#"SELECT
            id as "id!",
            user_id as "user_id!",
            parent_folder_id,
            is_file as "is_file!",
            name as "name!",
            "visibility!: FileVisibility",
            created_at as "created_at!",
            updated_at
        FROM (
            (
                SELECT
                    id,
                    user_id,
                    parent_folder_id,
                    FALSE as is_file,
                    name,
                    visibility as "visibility!: FileVisibility",
                    created_at,
                    updated_at
                FROM folders
                WHERE user_id = $1 AND (($2::uuid IS NULL AND parent_folder_id IS NULL) OR parent_folder_id = $2)
                ORDER BY name ASC
            ) UNION ALL (
                SELECT
                    id,
                    user_id,
                    parent_folder_id,
                    TRUE as is_file,
                    name,
                    visibility as "visibility!: FileVisibility",
                    created_at,
                    updated_at
                FROM files
                WHERE user_id = $1 AND (($2::uuid IS NULL AND parent_folder_id IS NULL) OR parent_folder_id = $2)
                ORDER BY name ASC
            )
        )"#,
        user.id,          // $1
        parent_folder_id, // $2
    )
    .fetch_all(db_pool)
    .await
    .unwrap_or_default()
}

pub async fn get_folder_by_id<'a>(id: Uuid, user: Option<&User<'_>>) -> sqlx::Result<Folder<'a>> {
    let db_pool = db_pool().await;
    let user_id = user.map(|u| u.id);

    sqlx::query_as!(
        Folder,
        r#"SELECT
            id, user_id, parent_folder_id, name, visibility as "visibility!: FileVisibility", created_at, updated_at
        FROM folders WHERE id = $1 AND ($2::uuid IS NULL OR user_id = $2) LIMIT 1"#,
        id,      // $1
        user_id, // $2
    )
    .fetch_one(db_pool)
    .await
}

pub async fn get_used_storage_by_user(user: &User<'_>) -> ByteSize {
    let db_pool = db_pool().await;

    sqlx::query!(
        r#"SELECT SUM(byte_size)::bigint AS "used_storage!" FROM files WHERE user_id = $1"#,
        user.id
    )
    .fetch_one(db_pool)
    .await
    .map(|row| ByteSize(row.used_storage as u64))
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

pub async fn get_user_session_by_id(id: uuid::Uuid) -> sqlx::Result<UserSession> {
    let db_pool = db_pool().await;

    sqlx::query_as!(UserSession, "SELECT * FROM user_sessions WHERE id = $1 LIMIT 1", id)
        .fetch_one(db_pool)
        .await
}

pub async fn insert_file<'a>(user: &User<'_>, input: &FileInput) -> Result<File<'a>, ValidationErrors> {
    input.validate()?;

    let mut validation_errors = ValidationErrors::new();
    let db_pool = db_pool().await;

    let mut md5_hasher = Md5::new();
    let mut visibility = FileVisibility::Private;
    let byte_size = input.content.len();
    let file_format = FileFormat::from_bytes(&input.content);
    let available_storage = user.available_storage().await;

    if file_name_exists(user, input.parent_folder_id, &input.name).await {
        validation_errors.add("name", ERROR_ALREADY_EXISTS.clone());
    }

    if let Some(parent_folder_id) = input.parent_folder_id {
        if let Ok(parent_folder) = get_folder_by_id(parent_folder_id, Some(user)).await {
            visibility = parent_folder.visibility;
        } else {
            validation_errors.add("parent_folder_id", ERROR_IS_INVALID.clone());
        }
    }

    if available_storage < ByteSize(byte_size as u64) {
        validation_errors.add("content", ERROR_IS_TOO_LARGE.clone());
    } else if !ALLOWED_FILE_FORMATS.contains(&file_format) {
        validation_errors.add("content", ERROR_IS_INVALID.clone());
    }

    if !validation_errors.is_empty() {
        return Err(validation_errors);
    }

    md5_hasher.update(&input.content);

    let md5_checksum = format!("{:x}", md5_hasher.finalize());

    let result = sqlx::query_as!(
        File,
        r#"INSERT INTO files (user_id, parent_folder_id, name, visibility, media_type, byte_size, md5_checksum)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING
            id,
            user_id,
            parent_folder_id,
            name,
            visibility as "visibility!: FileVisibility",
            media_type,
            byte_size,
            md5_checksum,
            created_at,
            updated_at"#,
        user.id,                  // $1
        input.parent_folder_id,   // $2
        input.name,               // $3
        visibility as _,          // $4
        file_format.media_type(), // $5
        byte_size as i64,         // $6
        md5_checksum,             // $7
    )
    .fetch_one(db_pool)
    .await;

    match result {
        Ok(file) => {
            let _ = std::fs::create_dir_all(file.directory());
            let mut fs_file = FsFile::create(file.default_path()).unwrap();

            let _ = fs_file.write_all(&input.content);

            Ok(file)
        }
        Err(_) => Err(ValidationErrors::new()),
    }
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
            id, user_id, parent_folder_id, name, visibility as "visibility!: FileVisibility", created_at, updated_at"#,
        user.id,                // $1
        input.parent_folder_id, // $2
        input.name,             // $3
        input.visibility as _   // $4
    )
    .fetch_one(db_pool)
    .await
    .map_err(|_| ValidationErrors::new())
}

pub async fn insert_user<'a>(input: &RegisterInput) -> Result<User<'a>, ValidationErrors> {
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
            country_alpha2,
            membership_code
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *",
        input.username,                    // $1
        input.email.to_lowercase(),        // $2
        encrypt_password(&input.password), // $3
        display_name,                      // $4
        input.full_name,                   // $5
        birthdate,                         // $6
        input.country_alpha2,              // $7
        MEMBERSHIPS_CONFIG.default,        // $8
    )
    .fetch_one(db_pool)
    .await
    .map_err(|_| ValidationErrors::new())
}

pub async fn insert_user_session(user: &User<'_>) -> sqlx::Result<UserSession> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        UserSession,
        "INSERT INTO user_sessions (user_id) VALUES ($1) RETURNING *",
        user.id, // $1
    )
    .fetch_one(db_pool)
    .await
}

pub async fn update_user_membership(user: &User<'_>, code: &str, is_annual: bool) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    if user.membership_code == code && user.membership_is_annual == is_annual {
        return Ok(());
    }

    if !MEMBERSHIPS_CONFIG.options.iter().any(|option| option.code == code) {
        return Err(sqlx::Error::InvalidArgument("code".to_owned()));
    }

    sqlx::query!(
        "UPDATE users SET membership_code = $2, membership_is_annual = $3, membership_updated_at = current_timestamp
        WHERE id = $1",
        user.id,   // $1
        code,      // $2
        is_annual, // $3
    )
    .execute(db_pool)
    .await
    .map(|_| ())
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
    async fn should_get_seven_folders_by_user() {
        let user = insert_test_user(None).await;

        insert_test_folders(7, Some(&user), None).await;

        let folders = get_all_folder_items_by_user(&user, None).await;

        assert_eq!(folders.len(), 7);
    }

    #[tokio::test]
    async fn should_get_seven_folders_by_user_with_parent_folder() {
        let user = insert_test_user(None).await;
        let parent_folder = insert_test_folder(Some(&user), None).await;

        insert_test_folders(7, Some(&user), Some(&parent_folder)).await;

        let folders = get_all_folder_items_by_user(&user, Some(&parent_folder)).await;

        assert_eq!(folders.len(), 7);
    }

    #[tokio::test]
    async fn should_get_zero_folders_by_user() {
        let user = insert_test_user(None).await;
        let folders = get_all_folder_items_by_user(&user, None).await;

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
    async fn should_insert_a_file() {
        let user = insert_test_user(None).await;
        let input = FileInput {
            parent_folder_id: None,
            name: fake_name() + ".jpg",
            content: vec![0xFF, 0xD8, 0xFF],
        };

        let result = insert_file(&user, &input).await;

        assert!(result.is_ok());

        let file = result.unwrap();

        assert_eq!(file.user_id, user.id);
        assert!(file.parent_folder_id.is_none());
        assert_eq!(file.name, input.name);
        assert_eq!(file.media_type, "image/jpeg")
    }

    #[tokio::test]
    async fn should_not_insert_an_invalid_file() {
        let user = insert_test_user(None).await;
        let input = FileInput {
            parent_folder_id: None,
            name: fake_name() + ".jpg",
            content: vec![],
        };

        let result = insert_file(&user, &input).await;

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

    #[tokio::test]
    async fn should_get_used_storage_equal_zero_when_is_empty() {
        let user = insert_test_user(None).await;

        let used_storage = get_used_storage_by_user(&user).await;

        assert_eq!(used_storage, ByteSize(0));
    }

    #[tokio::test]
    async fn should_get_used_storage_more_than_zero_after_insert() {
        let user = insert_test_user(None).await;

        insert_test_files(7, Some(&user)).await;

        let used_storage = get_used_storage_by_user(&user).await;

        assert!(used_storage > ByteSize(0));
    }
}
