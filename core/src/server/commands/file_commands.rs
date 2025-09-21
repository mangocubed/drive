use validator::{Validate, ValidationErrors};

use crate::inputs::RenameInput;
use crate::server::constants::ERROR_ALREADY_EXISTS;
use crate::server::db_pool;
use crate::server::models::File;

use super::file_name_exists;

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
