use validator::{Validate, ValidationErrors};

use crate::inputs::RenameInput;
use crate::server::constants::ERROR_ALREADY_EXISTS;
use crate::server::db_pool;
use crate::server::models::Folder;

use super::file_name_exists;

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
