use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

#[cfg(feature = "server")]
use axum_extra::TypedHeader;
#[cfg(not(feature = "server"))]
use dioxus::fullstack::client::Client;
#[cfg(feature = "server")]
use dioxus::fullstack::mock_client::MockServerFnClient;
#[cfg(not(feature = "server"))]
use futures::{Sink, Stream};
#[cfg(feature = "server")]
use headers::Authorization;
#[cfg(feature = "server")]
use headers::authorization::Bearer;
#[cfg(feature = "server")]
use serde_json::Value;
#[cfg(feature = "web")]
use server_fn::client::browser::BrowserClient;
#[cfg(any(feature = "desktop", feature = "mobile"))]
use server_fn::client::reqwest::ReqwestClient;
#[cfg(not(feature = "server"))]
use server_fn::error::FromServerFnError;
#[cfg(feature = "web")]
use server_fn::request::browser::BrowserRequest;
#[cfg(feature = "web")]
use server_fn::response::browser::BrowserResponse;

use drive_core::inputs::{FileInput, FolderInput};

#[cfg(feature = "server")]
use drive_core::server::models::{AccessToken, User};

use crate::hooks::FormStatus;
use crate::presenters::{FilePresenter, FolderItemPresenter, FolderPresenter, PlanPresenter, UserPresenter};
use crate::routes::Routes;

#[cfg(feature = "server")]
use crate::presenters::AsyncInto;

mod login_server_fns;
mod trash_server_fns;

pub use login_server_fns::*;
pub use trash_server_fns::*;

#[cfg(feature = "server")]
pub type ServFnClient = MockServerFnClient;

#[cfg(not(feature = "server"))]
pub struct ServFnClient;

#[cfg(feature = "web")]
impl<E, IS, OS> Client<E, IS, OS> for ServFnClient
where
    E: FromServerFnError,
    IS: FromServerFnError,
    OS: FromServerFnError,
{
    type Request = BrowserRequest;
    type Response = BrowserResponse;

    fn send(req: Self::Request) -> impl Future<Output = Result<Self::Response, E>> + Send {
        use crate::utils::{DataStorageTrait, data_storage};

        let headers = req.headers();
        let access_token = data_storage().get_access_token();

        if let Some(token) = access_token {
            headers.append("Authorization", &format!("Bearer {token}",));
        }

        <BrowserClient as Client<E, IS, OS>>::send(req)
    }

    fn open_websocket(
        path: &str,
    ) -> impl Future<
        Output = Result<
            (
                impl Stream<Item = Result<server_fn::Bytes, server_fn::Bytes>> + Send + 'static,
                impl Sink<server_fn::Bytes> + Send + 'static,
            ),
            E,
        >,
    > + Send {
        <BrowserClient as Client<E, IS, OS>>::open_websocket(path)
    }

    fn spawn(future: impl Future<Output = ()> + Send + 'static) {
        <BrowserClient as Client<E, IS, OS>>::spawn(future)
    }
}

#[cfg(any(feature = "desktop", feature = "mobile"))]
impl<E, IS, OS> Client<E, IS, OS> for ServFnClient
where
    E: FromServerFnError,
    IS: FromServerFnError,
    OS: FromServerFnError,
{
    type Request = reqwest::Request;
    type Response = reqwest::Response;

    fn send(mut req: Self::Request) -> impl Future<Output = Result<Self::Response, E>> + Send {
        use crate::utils::{DataStorageTrait, data_storage};

        let headers = req.headers_mut();
        let access_token = data_storage().get_access_token();

        if let Some(token) = access_token {
            headers.append("Authorization", format!("Bearer {token}").parse().unwrap());
        }

        <ReqwestClient as Client<E, IS, OS>>::send(req)
    }

    fn open_websocket(
        path: &str,
    ) -> impl Future<
        Output = Result<
            (
                impl Stream<Item = Result<server_fn::Bytes, server_fn::Bytes>> + Send + 'static,
                impl Sink<server_fn::Bytes> + Send + 'static,
            ),
            E,
        >,
    > + Send {
        <ReqwestClient as Client<E, IS, OS>>::open_websocket(path)
    }

    fn spawn(future: impl Future<Output = ()> + Send + 'static) {
        <ReqwestClient as Client<E, IS, OS>>::spawn(future)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServFnError {
    LoginRequired,
    NoLoginRequired,
    Other(String),
}

pub type ServFnResult<T = ()> = ServerFnResult<T, ServFnError>;

#[cfg(feature = "server")]
impl From<ServFnError> for ServerFnError<ServFnError> {
    fn from(value: ServFnError) -> Self {
        ServerFnError::ServerError(value)
    }
}

impl ServFnError {
    pub fn run_action(&self) {
        let navigator = use_navigator();

        match self {
            ServFnError::LoginRequired => {
                navigator.push(Routes::login());
            }
            ServFnError::NoLoginRequired => {
                navigator.push(Routes::home());
            }
            _ => (),
        }
    }
}

#[server(client = ServFnClient)]
pub async fn attempt_to_confirm_plan_checkout(checkout_id: Uuid) -> ServFnResult<String> {
    let result = drive_core::server::commands::confirm_plan_checkout(checkout_id).await;

    match result {
        Ok(_) => Ok("Subscription upgraded successfully".to_owned()),
        Err(errors) => Err(ServFnError::Other(errors.to_string()).into()),
    }
}

#[server(client = ServFnClient)]
pub async fn attempt_to_create_folder(input: FolderInput) -> ServFnResult<FormStatus> {
    require_login().await?;

    let user = extract_user().await?.unwrap();

    let result = drive_core::server::commands::insert_folder(&user, &input).await;

    match result {
        Ok(_) => Ok(FormStatus::Success(
            "Folder created successfully".to_owned(),
            Value::Null,
        )),
        Err(errors) => Ok(FormStatus::Failed("Failed to create folder".to_owned(), errors)),
    }
}

#[server(client = ServFnClient)]
pub async fn attempt_to_create_plan_checkout(plan_id: Uuid, is_yearly: bool) -> ServFnResult<Url> {
    require_login().await?;

    let user = extract_user().await?.unwrap();
    let plan = drive_core::server::commands::get_plan_by_id(plan_id)
        .await
        .map_err(|_| ServFnError::Other("Could not get plan".to_owned()))?;

    let result = drive_core::server::commands::create_user_plan_checkout(&user, &plan, is_yearly).await;

    match result {
        Ok(checkout) => Ok(checkout.url),
        Err(error) => Err(ServFnError::Other(error.to_string()).into()),
    }
}

#[server(client = ServFnClient)]
pub async fn attempt_to_upload_file(input: FileInput) -> ServFnResult<bool> {
    require_login().await?;

    let user = extract_user().await?.unwrap();

    let result = drive_core::server::commands::insert_file(&user, &input).await;

    Ok(result.is_ok())
}

#[cfg(feature = "server")]
async fn extract_bearer() -> ServFnResult<Option<Bearer>> {
    if let Some(TypedHeader(Authorization(bearer))) = extract::<Option<TypedHeader<Authorization<Bearer>>>, _>()
        .await
        .map_err(|error| ServFnError::Other(error.to_string()))?
    {
        Ok(Some(bearer))
    } else {
        Ok(None)
    }
}

#[cfg(feature = "server")]
async fn extract_access_token<'a>() -> ServFnResult<Option<AccessToken<'a>>> {
    if let Some(bearer) = extract_bearer().await? {
        Ok(drive_core::server::commands::get_access_token(bearer.token())
            .await
            .ok())
    } else {
        Ok(None)
    }
}

#[cfg(feature = "server")]
async fn extract_user<'a>() -> ServFnResult<Option<User<'a>>> {
    if let Some(bearer) = extract_bearer().await? {
        Ok(drive_core::server::commands::get_user_by_access_token(bearer.token())
            .await
            .ok())
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
            drive_core::server::commands::get_folder_by_id(id, Some(&user))
                .await
                .map_err(|_| ServFnError::Other("Could not get parent folder".to_owned()))?,
        )
    } else {
        None
    };
    let folder_items = drive_core::server::commands::get_all_folder_items(Some(&user), parent_folder.as_ref())
        .await
        .map_err(|_| ServFnError::Other("Could not get folder items".to_owned()))?;

    Ok(futures::future::join_all(folder_items.iter().map(|folder_item| folder_item.async_into())).await)
}

#[server(client = ServFnClient)]
pub async fn get_current_user() -> ServFnResult<Option<UserPresenter>> {
    let Some(user) = extract_user().await? else {
        return Ok(None);
    };

    Ok(Some(user.async_into().await))
}

#[server(client = ServFnClient)]
pub async fn get_file(id: Uuid) -> ServFnResult<Option<FilePresenter>> {
    require_login().await?;

    let user = extract_user().await?.unwrap();

    let result = drive_core::server::commands::get_file_by_id(id, Some(&user)).await;

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

    let result = drive_core::server::commands::get_folder_by_id(id, Some(&user)).await;

    Ok(if let Ok(folder) = result {
        Some(folder.async_into().await)
    } else {
        None
    })
}

#[server(client = ServFnClient)]
pub async fn get_all_available_plans() -> ServFnResult<Vec<PlanPresenter>> {
    require_login().await?;

    Ok(drive_core::server::commands::get_all_plans()
        .await
        .map_err(|_| ServFnError::Other("Could not get plans".to_owned()))?
        .iter()
        .map(|plan| plan.into())
        .collect())
}

#[server(client = ServFnClient)]
pub async fn is_logged_in() -> ServFnResult<bool> {
    Ok(extract_access_token().await?.is_some())
}

#[cfg(feature = "server")]
async fn require_login() -> ServFnResult<()> {
    if !is_logged_in().await? {
        Err(ServerFnError::ServerError(ServFnError::LoginRequired))
    } else {
        Ok(())
    }
}

#[cfg(feature = "server")]
async fn require_no_login() -> ServFnResult<()> {
    if is_logged_in().await? {
        Err(ServerFnError::ServerError(ServFnError::NoLoginRequired))
    } else {
        Ok(())
    }
}
