use validator::{Validate, ValidationErrors};

use crate::inputs::RenameInput;
use crate::server::db_pool;
use crate::server::models::File;

pub async fn rename_file(file: &File<'_>, input: &RenameInput) -> Result<(), ValidationErrors> {
    if input.name == file.name {
        return Ok(());
    }

    input.validate()?;

    let db_pool = db_pool().await;

    sqlx::query!("UPDATE files SET name = $1 WHERE id = $2", input.name, file.id)
        .execute(db_pool)
        .await
        .map(|_| ())
        .map_err(|_| ValidationErrors::new())
}
