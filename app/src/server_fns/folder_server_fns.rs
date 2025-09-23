use dioxus::prelude::*;
use uuid::Uuid;

use drive_core::inputs::RenameInput;

#[cfg(feature = "server")]
use serde_json::Value;

use crate::hooks::FormStatus;

use super::{ServFnClient, ServFnResult};

#[cfg(feature = "server")]
use super::{ServFnError, extract_user, require_login};

#[server(client = ServFnClient)]
pub async fn attempt_to_move_folder(folder_id: Uuid, target_folder_id: Option<Uuid>) -> ServFnResult<()> {
    require_login().await?;

    let user = extract_user().await?.unwrap();
    let folder = drive_core::server::commands::get_folder_by_id(folder_id, Some(&user))
        .await
        .map_err(|_| ServFnError::Other("Could not get folder".to_owned()))?;
    let target_folder = if let Some(target_folder_id) = target_folder_id {
        Some(
            &drive_core::server::commands::get_folder_by_id(target_folder_id, Some(&user))
                .await
                .map_err(|_| ServFnError::Other("Could not get target folder".to_owned()))?,
        )
    } else {
        None
    };

    drive_core::server::commands::move_folder(&folder, target_folder)
        .await
        .map_err(|_| ServFnError::Other("Could not move file".to_owned()))?;

    Ok(())
}

#[server(client = ServFnClient)]
pub async fn attempt_to_rename_folder(input: RenameInput) -> ServFnResult<FormStatus> {
    require_login().await?;

    let user = extract_user().await?.unwrap();
    let folder = drive_core::server::commands::get_folder_by_id(input.id, Some(&user))
        .await
        .map_err(|_| ServFnError::Other("Could not get folder".to_owned()))?;

    let result = drive_core::server::commands::rename_folder(&folder, &input).await;

    match result {
        Ok(_) => Ok(FormStatus::Success(
            "Folder renamed successfully".to_owned(),
            Value::Null,
        )),
        Err(errors) => Ok(FormStatus::Failed("Failed to rename folder".to_owned(), errors)),
    }
}
