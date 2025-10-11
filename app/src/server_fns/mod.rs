use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use url::Url;
use uuid::Uuid;

#[cfg(feature = "server")]
use serde_json::Value;

use sdk::serv_fn::{FormResult, ServFnClient, ServFnResult};

#[cfg(feature = "server")]
use sdk::serv_fn::{FormError, FormSuccess, ServFnError, extract_bearer, require_app_token};

use drive_core::inputs::{FileInput, FolderInput};

#[cfg(feature = "server")]
use drive_core::server::commands;
#[cfg(feature = "server")]
use drive_core::server::models::{Session, User};

use crate::presenters::{FilePresenter, FolderItemPresenter, FolderPresenter, PlanPresenter, UserPresenter};

#[cfg(feature = "server")]
use crate::presenters::AsyncInto;

mod file_server_fns;
mod folder_server_fns;
mod trash_server_fns;

pub use file_server_fns::*;
pub use folder_server_fns::*;
pub use trash_server_fns::*;

#[server(client = ServFnClient)]
pub async fn attempt_to_confirm_authorization(token: String, expires_at: DateTime<Utc>) -> ServFnResult<String> {
    require_no_login().await?;

    let result = commands::confirm_authorization(&token, expires_at).await;

    match result {
        Ok(session) => Ok(session.token.to_string()),
        Err(_) => Err(ServFnError::bad_request().into()),
    }
}

#[server(client = ServFnClient)]
pub async fn attempt_to_confirm_plan_checkout(checkout_id: Uuid) -> ServFnResult<String> {
    let result = commands::confirm_plan_checkout(checkout_id).await;

    match result {
        Ok(_) => Ok("Subscription upgraded successfully".to_owned()),
        Err(_) => Err(ServFnError::bad_request().into()),
    }
}

#[server(client = ServFnClient)]
pub async fn attempt_to_create_folder(input: FolderInput) -> FormResult {
    require_login().await.map_err(FormError::from)?;

    let user = extract_user().await.map_err(FormError::from)?.unwrap();

    let result = commands::insert_folder(&user, &input).await;

    match result {
        Ok(_) => Ok(FormSuccess::new("Folder created successfully", Value::Null)),
        Err(errors) => Err(FormError::new("Failed to create folder", Some(errors)).into()),
    }
}

#[server(client = ServFnClient)]
pub async fn attempt_to_create_plan_checkout(plan_id: Uuid, is_yearly: bool) -> ServFnResult<Url> {
    require_login().await?;

    let user = extract_user().await?.unwrap();
    let plan = commands::get_plan_by_id(plan_id)
        .await
        .map_err(|_| ServFnError::bad_request())?;

    let result = commands::create_user_plan_checkout(&user, &plan, is_yearly).await;

    match result {
        Ok(checkout) => Ok(checkout.url),
        Err(_) => Err(ServFnError::bad_request().into()),
    }
}

#[server(client = ServFnClient)]
pub async fn attempt_to_upload_file(input: FileInput) -> ServFnResult<bool> {
    require_login().await?;

    let user = extract_user().await?.unwrap();

    let result = commands::insert_file(&user, &input).await;

    Ok(result.is_ok())
}

#[cfg(feature = "server")]
async fn extract_session<'a>() -> ServFnResult<Option<Session<'a>>> {
    if let Some(bearer) = extract_bearer().await? {
        Ok(commands::get_session_by_token(bearer.token()).await.ok())
    } else {
        Ok(None)
    }
}

#[cfg(feature = "server")]
async fn extract_user<'a>() -> ServFnResult<Option<User<'a>>> {
    if let Some(bearer) = extract_bearer().await? {
        Ok(commands::get_user_by_session_token(bearer.token()).await.ok())
    } else {
        Ok(None)
    }
}

#[server(client = ServFnClient)]
pub async fn get_all_folder_items(parent_folder_id: Option<Uuid>) -> ServFnResult<Vec<FolderItemPresenter>> {
    require_login().await?;

    let user = extract_user().await?.unwrap();
    let parent_folder = if let Some(id) = parent_folder_id {
        Some(
            commands::get_folder_by_id(id, Some(&user))
                .await
                .map_err(|_| ServFnError::bad_request())?,
        )
    } else {
        None
    };
    let folder_items = commands::get_all_folder_items(Some(&user), parent_folder.as_ref())
        .await
        .expect("Could not get folder items");

    Ok(futures::future::join_all(folder_items.iter().map(|folder_item| folder_item.async_into())).await)
}

#[server(client = ServFnClient)]
pub async fn get_current_user() -> ServFnResult<Option<UserPresenter>> {
    require_app_token().await?;

    let Some(user) = extract_user().await? else {
        return Ok(None);
    };

    Ok(Some(user.async_into().await))
}

#[server(client = ServFnClient)]
pub async fn get_file(id: Uuid) -> ServFnResult<Option<FilePresenter>> {
    require_login().await?;

    let user = extract_user().await?.unwrap();

    let result = commands::get_file_by_id(id, Some(&user)).await;

    Ok(if let Ok(file) = result {
        Some(file.async_into().await)
    } else {
        None
    })
}

#[server(client = ServFnClient)]
pub async fn get_folder(id: Uuid) -> ServFnResult<Option<FolderPresenter>> {
    require_login().await?;

    let user = extract_user().await?.unwrap();

    let result = commands::get_folder_by_id(id, Some(&user)).await;

    Ok(if let Ok(folder) = result {
        Some(folder.async_into().await)
    } else {
        None
    })
}

#[server(client = ServFnClient)]
pub async fn get_all_available_plans() -> ServFnResult<Vec<PlanPresenter>> {
    require_login().await?;

    Ok(commands::get_all_plans()
        .await
        .expect("Could not get plans")
        .iter()
        .map(|plan| plan.into())
        .collect())
}

#[server(client = ServFnClient)]
pub async fn is_logged_in() -> ServFnResult<bool> {
    require_app_token().await?;

    Ok(extract_session().await?.is_some())
}

#[cfg(feature = "server")]
async fn require_login() -> ServFnResult<()> {
    if is_logged_in().await? {
        Ok(())
    } else {
        Err(ServFnError::unauthorized().into())
    }
}

#[cfg(feature = "server")]
async fn require_no_login() -> ServFnResult<()> {
    if !is_logged_in().await? {
        Ok(())
    } else {
        Err(ServFnError::forbidden().into())
    }
}
