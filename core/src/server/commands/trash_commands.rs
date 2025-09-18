use crate::enums::FileVisibility;
use crate::server::db_pool;
use crate::server::models::{File, Folder, FolderItem, User};

use super::get_all_folder_items;

pub async fn delete_file(file: &File<'_>) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!("DELETE FROM files WHERE id = $1", file.id)
        .execute(db_pool)
        .await?;

    let _ = std::fs::remove_file(file.default_path());

    Ok(())
}

pub async fn delete_folder(folder: &Folder<'_>) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    let folder_items = get_all_folder_items(None, Some(folder)).await;

    if let Ok(folder_items) = folder_items {
        futures::future::join_all(folder_items.iter().map(|item| async {
            if item.is_file {
                delete_file(&item.into()).await
            } else {
                delete_folder(&item.into()).await
            }
        }))
        .await;
    }

    sqlx::query!(
        "UPDATE folders SET parent_folder_id = NULL WHERE trashed_at IS NOT NULL AND parent_folder_id = $1",
        folder.id
    )
    .execute(db_pool)
    .await?;

    sqlx::query!(
        "UPDATE files SET parent_folder_id = NULL WHERE trashed_at IS NOT NULL AND parent_folder_id = $1",
        folder.id
    )
    .execute(db_pool)
    .await?;

    sqlx::query!("DELETE FROM folders WHERE id = $1", folder.id)
        .execute(db_pool)
        .await?;

    Ok(())
}

pub async fn empty_trash(user: &User<'_>) -> sqlx::Result<()> {
    let trash_items = get_all_trash_items(user).await?;

    futures::future::join_all(trash_items.iter().map(|item| async {
        if item.is_file {
            delete_file(&item.into()).await
        } else {
            delete_folder(&item.into()).await
        }
    }))
    .await;

    Ok(())
}

pub async fn folder_is_trashed(folder: &Folder<'_>) -> bool {
    if folder.trashed_at.is_some() {
        return true;
    }

    if let Some(parent_folder_id) = folder.parent_folder_id {
        let db_pool = db_pool().await;

        return sqlx::query!(
            "WITH RECURSIVE parent_folders AS (
                SELECT id, parent_folder_id, trashed_at FROM folders WHERE id = $1
                UNION ALL
                SELECT f.id, f.parent_folder_id, f.trashed_at FROM folders as f, parent_folders AS pf
                WHERE f.id = pf.parent_folder_id
            ) SELECT id FROM parent_folders WHERE trashed_at IS NOT NULL LIMIT 1",
            parent_folder_id
        )
        .fetch_one(db_pool)
        .await
        .is_ok();
    }

    false
}

pub async fn get_all_trash_items<'a>(user: &User<'_>) -> sqlx::Result<Vec<FolderItem<'a>>> {
    let db_pool = db_pool().await;

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
                FROM folders WHERE user_id = $1 AND trashed_at IS NOT NULL ORDER BY name ASC
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
                FROM files WHERE user_id = $1 AND trashed_at IS NOT NULL ORDER BY name ASC
            )
        )"#,
        user.id, // $1
    )
    .fetch_all(db_pool)
    .await
}

pub async fn move_file_to_trash(file: &File<'_>) -> sqlx::Result<()> {
    if file.trashed_at.is_some() {
        return Ok(());
    }

    let db_pool = db_pool().await;

    sqlx::query!("UPDATE files SET trashed_at = current_timestamp WHERE id = $1", file.id)
        .execute(db_pool)
        .await
        .map(|_| ())
}

pub async fn move_folder_to_trash(folder: &Folder<'_>) -> sqlx::Result<()> {
    if folder.trashed_at.is_some() {
        return Ok(());
    }

    let db_pool = db_pool().await;

    sqlx::query!(
        "UPDATE folders SET trashed_at = current_timestamp WHERE id = $1",
        folder.id
    )
    .execute(db_pool)
    .await
    .map(|_| ())
}

pub async fn restore_file(file: &File<'_>) -> sqlx::Result<()> {
    if file.trashed_at.is_none() {
        return Ok(());
    }

    let db_pool = db_pool().await;

    let parent_folder_id = if let Some(parent_folder) = file.parent_folder().await
        && !parent_folder.is_trashed().await
    {
        file.parent_folder_id
    } else {
        None
    };

    sqlx::query!(
        "UPDATE files SET trashed_at = NULL, parent_folder_id = $2 WHERE id = $1",
        file.id,          // $1
        parent_folder_id  // $2
    )
    .execute(db_pool)
    .await
    .map(|_| ())
}

pub async fn restore_folder(folder: &Folder<'_>) -> sqlx::Result<()> {
    if folder.trashed_at.is_none() {
        return Ok(());
    }

    let db_pool = db_pool().await;

    let parent_folder_id = if let Some(parent_folder) = folder.parent_folder().await
        && !parent_folder.is_trashed().await
    {
        folder.parent_folder_id
    } else {
        None
    };

    sqlx::query!(
        "UPDATE folders SET parent_folder_id = $2, trashed_at = NULL WHERE id = $1",
        folder.id,        // $1
        parent_folder_id  // $2
    )
    .execute(db_pool)
    .await
    .map(|_| ())
}
