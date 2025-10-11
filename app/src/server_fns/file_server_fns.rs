use dioxus::prelude::*;
use url::Url;
use uuid::Uuid;

#[cfg(feature = "server")]
use serde_json::Value;

use drive_core::inputs::RenameInput;

#[cfg(feature = "server")]
use drive_core::server::commands;

use sdk::serv_fn::{FormResult, ServFnClient, ServFnResult};

#[cfg(feature = "server")]
use sdk::serv_fn::{FormError, FormSuccess, ServFnError};

#[cfg(feature = "server")]
use super::{extract_user, require_login};

#[server(client = ServFnClient)]
pub async fn attempt_to_move_file(file_id: Uuid, target_folder_id: Option<Uuid>) -> ServFnResult {
    require_login().await?;

    let user = extract_user().await?.unwrap();
    let file = commands::get_file_by_id(file_id, Some(&user))
        .await
        .map_err(|_| ServFnError::not_found())?;
    let target_folder = if let Some(target_folder_id) = target_folder_id {
        Some(
            commands::get_folder_by_id(target_folder_id, Some(&user))
                .await
                .map_err(|_| ServFnError::bad_request())?,
        )
    } else {
        None
    };

    commands::move_file(&file, target_folder.as_ref())
        .await
        .map_err(|_| ServFnError::bad_request())?;

    Ok(())
}

#[server(client = ServFnClient)]
pub async fn attempt_to_rename_file(input: RenameInput) -> FormResult {
    require_login().await.map_err(FormError::from)?;

    let user = extract_user().await.map_err(FormError::from)?.unwrap();
    let file = commands::get_file_by_id(input.id, Some(&user))
        .await
        .map_err(|_| FormError::new("Failed to rename file", None))?;

    let result = commands::rename_file(&file, &input).await;

    match result {
        Ok(_) => Ok(FormSuccess::new("File renamed successfully", Value::Null)),
        Err(errors) => Err(FormError::new("Failed to rename file", Some(errors)).into()),
    }
}

#[server(client = ServFnClient)]
pub async fn get_file_url(id: Uuid) -> ServFnResult<Url> {
    require_login().await?;

    let user = extract_user().await?.unwrap();
    let file = drive_core::server::commands::get_file_by_id(id, Some(&user))
        .await
        .map_err(|_| ServFnError::not_found())?;

    Ok(file.url().await)
}
