use uuid::Uuid;
use validator::{Validate, ValidationErrors};

use crate::enums::FileVisibility;
use crate::inputs::RenameInput;
use crate::server::constants::ERROR_ALREADY_EXISTS;
use crate::server::db_pool;
use crate::server::models::Folder;

use super::file_name_exists;

pub async fn get_folder_parent_folders<'a>(folder: &Folder<'_>) -> sqlx::Result<Vec<Folder<'a>>> {
    get_parent_folders_by_id(folder.parent_folder_id).await
}

pub async fn get_parent_folders_by_id<'a>(id: Option<Uuid>) -> sqlx::Result<Vec<Folder<'a>>> {
    if id.is_none() {
        return Ok(vec![]);
    }

    let db_pool = db_pool().await;

    sqlx::query_as!(
        Folder,
        r#"WITH RECURSIVE parent_folders AS (
            SELECT * FROM folders WHERE id = $1
            UNION ALL
            SELECT f.* FROM folders as f, parent_folders AS pf WHERE f.id = pf.parent_folder_id
            ) SELECT
                id as "id!",
                user_id as "user_id!",
                parent_folder_id,
                name as "name!",
                visibility as "visibility!: FileVisibility",
                trashed_at,
                created_at as "created_at!",
                updated_at
            FROM parent_folders LIMIT 1"#,
        id
    )
    .fetch_all(db_pool)
    .await
    .map(|folders| folders.into_iter().rev().collect())
}

pub async fn move_folder(folder: &Folder<'_>, target_folder: Option<&Folder<'_>>) -> sqlx::Result<()> {
    let target_folder_id = target_folder.map(|tf| tf.id);

    if folder.parent_folder_id == target_folder_id {
        return Ok(());
    }

    let db_pool = db_pool().await;

    if let Some(target_folder) = target_folder {
        let is_invalid = folder.id == target_folder.id
            || Some(folder.id) == target_folder.parent_folder_id
            || sqlx::query!(
                "WITH RECURSIVE parent_folders AS (
                    SELECT id, parent_folder_id FROM folders WHERE id = $1
                    UNION ALL
                    SELECT f.id, f.parent_folder_id FROM folders as f, parent_folders AS pf
                    WHERE f.id = pf.parent_folder_id
                ) SELECT id FROM parent_folders WHERE id = $2 LIMIT 1",
                target_folder.id, // $1
                folder.id,        // $2
            )
            .fetch_one(db_pool)
            .await
            .is_ok();

        if is_invalid {
            return Err(sqlx::Error::InvalidArgument(
                "Cannot move folder into itself".to_owned(),
            ));
        }
    }

    sqlx::query!(
        "UPDATE folders SET parent_folder_id = $2 WHERE id = $1",
        folder.id,        // $1
        target_folder_id, // $2
    )
    .execute(db_pool)
    .await
    .map(|_| ())
}

pub async fn rename_folder(folder: &Folder<'_>, input: &RenameInput) -> Result<(), ValidationErrors> {
    input.validate()?;

    if input.name == folder.name {
        return Ok(());
    }

    if input.name.to_lowercase() != folder.name.to_lowercase() {
        let mut validation_errors = ValidationErrors::new();

        if file_name_exists(&folder.user().await, folder.parent_folder_id, &input.name).await {
            validation_errors.add("name", ERROR_ALREADY_EXISTS.clone());
        }

        if !validation_errors.is_empty() {
            return Err(validation_errors);
        }
    }

    let db_pool = db_pool().await;

    sqlx::query!("UPDATE folders SET name = $1 WHERE id = $2", input.name, folder.id)
        .execute(db_pool)
        .await
        .map(|_| ())
        .map_err(|_| ValidationErrors::new())
}
