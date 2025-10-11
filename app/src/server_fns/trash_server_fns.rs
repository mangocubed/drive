use dioxus::prelude::*;
use uuid::Uuid;

use sdk::serv_fn::{ServFnClient, ServFnResult};

#[cfg(feature = "server")]
use sdk::serv_fn::ServFnError;

#[cfg(feature = "server")]
use drive_core::server::commands;

use crate::presenters::FolderItemPresenter;

#[cfg(feature = "server")]
use crate::presenters::AsyncInto;

#[cfg(feature = "server")]
use super::{extract_user, require_login};

#[server(client = ServFnClient)]
pub async fn attempt_to_empty_trash() -> ServFnResult<()> {
    require_login().await?;

    let user = extract_user().await?.unwrap();

    drive_core::server::commands::empty_trash(&user)
        .await
        .map_err(|_| ServFnError::bad_request())?;

    Ok(())
}

#[server(client = ServFnClient)]
pub async fn attempt_to_move_file_to_trash(file_id: Uuid) -> ServFnResult<()> {
    require_login().await?;

    let user = extract_user().await?.unwrap();
    let file = drive_core::server::commands::get_file_by_id(file_id, Some(&user))
        .await
        .map_err(|_| ServFnError::not_found())?;

    drive_core::server::commands::move_file_to_trash(&file)
        .await
        .map_err(|_| ServFnError::bad_request())?;

    Ok(())
}

#[server(client = ServFnClient)]
pub async fn attempt_to_move_folder_to_trash(folder_id: Uuid) -> ServFnResult<()> {
    require_login().await?;

    let user = extract_user().await?.unwrap();
    let folder = drive_core::server::commands::get_folder_by_id(folder_id, Some(&user))
        .await
        .map_err(|_| ServFnError::not_found())?;

    drive_core::server::commands::move_folder_to_trash(&folder)
        .await
        .map_err(|_| ServFnError::bad_request())?;

    Ok(())
}

#[server(client = ServFnClient)]
pub async fn attempt_to_restore_file(file_id: Uuid) -> ServFnResult<()> {
    require_login().await?;

    let user = extract_user().await?.unwrap();
    let file = drive_core::server::commands::get_file_by_id(file_id, Some(&user))
        .await
        .map_err(|_| ServFnError::not_found())?;

    drive_core::server::commands::restore_file(&file)
        .await
        .map_err(|_| ServFnError::bad_request())?;

    Ok(())
}

#[server(client = ServFnClient)]
pub async fn attempt_to_restore_folder(folder_id: Uuid) -> ServFnResult<()> {
    require_login().await?;

    let user = extract_user().await?.unwrap();
    let folder = drive_core::server::commands::get_folder_by_id(folder_id, Some(&user))
        .await
        .map_err(|_| ServFnError::not_found())?;

    drive_core::server::commands::restore_folder(&folder)
        .await
        .map_err(|_| ServFnError::bad_request())?;

    Ok(())
}

#[server(client = ServFnClient)]
pub async fn get_all_trash_items() -> ServFnResult<Vec<FolderItemPresenter>> {
    require_login().await?;

    let user = extract_user().await?.unwrap();
    let trash_items = commands::get_all_trash_items(&user)
        .await
        .expect("Could not get trash items");

    Ok(futures::future::join_all(trash_items.iter().map(|trash_item| trash_item.async_into())).await)
}
