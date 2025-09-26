use std::fs::File as FsFile;
use std::io::Write;

use bytesize::ByteSize;
use file_format::FileFormat;
use md5::{Digest, Md5};
use url::Url;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

use crate::enums::FileVisibility;
use crate::inputs::{FileInput, RenameInput};
use crate::server::config::{APP_CONFIG, STORAGE_CONFIG};
use crate::server::constants::{ALLOWED_FILE_FORMATS, ERROR_ALREADY_EXISTS, ERROR_IS_INVALID, ERROR_IS_TOO_LARGE};
use crate::server::db_pool;
use crate::server::models::{File, FileKey, Folder, User};

use super::{file_name_exists, get_available_space, get_folder_by_id, get_parent_folders_by_id};

pub async fn get_file_parent_folders<'a>(file: &File<'_>) -> sqlx::Result<Vec<Folder<'a>>> {
    get_parent_folders_by_id(file.parent_folder_id).await
}

pub async fn get_file_key_by_id(id: Uuid) -> sqlx::Result<FileKey> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        FileKey,
        "SELECT * FROM file_keys WHERE id = $1 AND created_at > current_timestamp - MAKE_INTERVAL(secs => $2)
        LIMIT 1",
        id,                                           // $1
        STORAGE_CONFIG.file_key_duration_secs as f64  // $2
    )
    .fetch_one(db_pool)
    .await
}

pub async fn get_file_url(file: &File<'_>) -> anyhow::Result<Url> {
    let file_key = insert_file_key(file).await?;
    let file_url = APP_CONFIG.server_url.join(&format!("storage/files/{}", file_key.id))?;

    Ok(file_url)
}

pub async fn insert_file<'a>(user: &User<'_>, input: &FileInput) -> Result<File<'a>, ValidationErrors> {
    input.validate()?;

    let mut validation_errors = ValidationErrors::new();
    let db_pool = db_pool().await;

    let mut md5_hasher = Md5::new();
    let mut visibility = FileVisibility::Private;
    let byte_size = input.content.len();
    let file_format = FileFormat::from_bytes(&input.content);

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

    let file_size = ByteSize(byte_size as u64);

    let available_space = [
        STORAGE_CONFIG.max_size_per_file(),
        user.available_space().await,
        get_available_space(),
    ]
    .iter()
    .min()
    .cloned()
    .unwrap_or(ByteSize(0));

    if available_space < file_size {
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
            trashed_at,
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

pub async fn insert_file_key(file: &File<'_>) -> sqlx::Result<FileKey> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        FileKey,
        "INSERT INTO file_keys (file_id) VALUES ($1) RETURNING *",
        file.id, // $1
    )
    .fetch_one(db_pool)
    .await
}

pub async fn move_file(file: &File<'_>, target_folder: Option<&Folder<'_>>) -> sqlx::Result<()> {
    let target_folder_id = target_folder.map(|tf| tf.id);

    if file.parent_folder_id == target_folder_id {
        return Ok(());
    }

    let db_pool = db_pool().await;

    sqlx::query!(
        "UPDATE files SET parent_folder_id = $2 WHERE id = $1",
        file.id,          // $1
        target_folder_id, // $2
    )
    .execute(db_pool)
    .await
    .map(|_| ())
}

pub async fn rename_file(file: &File<'_>, input: &RenameInput) -> Result<(), ValidationErrors> {
    input.validate()?;

    if input.name == file.name {
        return Ok(());
    }

    if input.name.to_lowercase() != file.name.to_lowercase() {
        let mut validation_errors = ValidationErrors::new();

        if file_name_exists(&file.user().await, file.parent_folder_id, &input.name).await {
            validation_errors.add("name", ERROR_ALREADY_EXISTS.clone());
        }

        if !validation_errors.is_empty() {
            return Err(validation_errors);
        }
    }

    let db_pool = db_pool().await;

    sqlx::query!("UPDATE files SET name = $1 WHERE id = $2", input.name, file.id)
        .execute(db_pool)
        .await
        .map(|_| ())
        .map_err(|_| ValidationErrors::new())
}

#[cfg(test)]
mod tests {
    use crate::test_utils::*;

    use super::*;

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
}
