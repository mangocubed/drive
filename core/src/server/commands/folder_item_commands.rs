use crate::enums::FileVisibility;
use crate::server::db_pool;
use crate::server::models::{Folder, FolderItem, User};

pub async fn get_all_folder_items<'a>(
    user: Option<&User<'_>>,
    parent_folder: Option<&Folder<'_>>,
) -> sqlx::Result<Vec<FolderItem<'a>>> {
    let db_pool = db_pool().await;
    let user_id = user.map(|u| u.id);
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
                WHERE ($1::uuid IS NULL OR user_id = $1)
                    AND (($2::uuid IS NULL AND parent_folder_id IS NULL) OR parent_folder_id = $2)
                    AND trashed_at IS NULL
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
                WHERE ($1::uuid IS NULL OR user_id = $1)
                    AND (($2::uuid IS NULL AND parent_folder_id IS NULL) OR parent_folder_id = $2)
                    AND trashed_at IS NULL
                ORDER BY name ASC
            )
        )"#,
        user_id,          // $1
        parent_folder_id, // $2
    )
    .fetch_all(db_pool)
    .await
}
