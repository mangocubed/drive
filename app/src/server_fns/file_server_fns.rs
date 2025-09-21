use dioxus::prelude::*;

use drive_core::inputs::RenameInput;

#[cfg(feature = "server")]
use serde_json::Value;

use crate::hooks::FormStatus;

use super::{ServFnClient, ServFnResult};

#[cfg(feature = "server")]
use super::{ServFnError, extract_user, require_login};

#[server(client = ServFnClient)]
pub async fn attempt_to_rename_file(input: RenameInput) -> ServFnResult<FormStatus> {
    require_login().await?;

    let user = extract_user().await?.unwrap();
    let file = drive_core::server::commands::get_file_by_id(input.id, Some(&user))
        .await
        .map_err(|_| ServFnError::Other("Could not get file".to_owned()))?;

    let result = drive_core::server::commands::rename_file(&file, &input).await;

    match result {
        Ok(_) => Ok(FormStatus::Success("File renamed successfully".to_owned(), Value::Null)),
        Err(errors) => Ok(FormStatus::Failed("Failed to rename file".to_owned(), errors)),
    }
}
