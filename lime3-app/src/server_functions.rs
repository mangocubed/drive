use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "server")]
use tower_sessions::Session;
#[cfg(feature = "server")]
use validator::ValidationErrors;

use lime3_core::inputs::{FolderInput, LoginInput, RegisterInput};

#[cfg(feature = "server")]
use lime3_core::server::commands::*;
#[cfg(feature = "server")]
use lime3_core::server::models::{User, UserSession};

use crate::forms::FormStatus;
use crate::presenters::{FolderPresenter, UserPresenter};
use crate::routes::Routes;

#[cfg(feature = "server")]
use crate::presenters::AsyncInto;
#[cfg(feature = "server")]
use crate::server::SessionTrait;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServFnError {
    LoginRequired,
    NoLoginRequired,
    Other(String),
}

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

pub type ServFnResult<T = ()> = ServerFnResult<T, ServFnError>;

#[server]
pub async fn attempt_to_create_folder(input: FolderInput) -> ServFnResult<FormStatus> {
    require_login().await?;

    let user = extract_user().await?.unwrap();

    let result = insert_folder(&user, &input).await;

    match result {
        Ok(_) => Ok(FormStatus::Success("Folder created successfully".to_owned())),
        Err(errors) => Ok(FormStatus::Failed("Failed to create folder".to_owned(), errors)),
    }
}

#[server]
pub async fn attempt_to_login(input: LoginInput) -> ServFnResult<FormStatus> {
    require_no_login().await?;

    let user = {
        let result = authenticate_user(&input).await;

        match result {
            Ok(user) => user,
            Err(errors) => {
                return Ok(FormStatus::Failed("Failed to authenticate user".to_owned(), errors));
            }
        }
    };

    let result = insert_user_session(&user).await;

    match result {
        Ok(user_session) => {
            let session = extract_session().await?;

            let _ = session.set_user_session(user_session).await;

            Ok(FormStatus::Success("User authenticated successfully".to_owned()))
        }
        Err(_) => Ok(FormStatus::Failed(
            "Failed to authenticate user".to_owned(),
            ValidationErrors::new(),
        )),
    }
}

#[server]
pub async fn attempt_to_logout() -> ServFnResult<()> {
    require_login().await?;

    let Some(user_session) = extract_user_session().await? else {
        return Ok(());
    };

    let _ = delete_user_session(&user_session).await;

    let session = extract_session().await?;

    let _ = session.remove_user_session().await;

    Ok(())
}

#[server]
pub async fn attempt_to_register(input: RegisterInput) -> ServFnResult<FormStatus> {
    require_no_login().await?;

    let result = insert_user(&input).await;

    match result {
        Ok(user) => {
            let result = insert_user_session(&user).await;

            if let Ok(user_session) = result {
                let session = extract_session().await?;

                let _ = session.set_user_session(user_session).await;
            }

            Ok(FormStatus::Success("User created successfully".to_owned()))
        }
        Err(errors) => Ok(FormStatus::Failed("Failed to create user".to_owned(), errors)),
    }
}

#[cfg(feature = "server")]
async fn extract_session() -> ServFnResult<Session> {
    extract()
        .await
        .map_err(|_| ServFnError::Other("Session layer not found".to_owned()).into())
}

#[cfg(feature = "server")]
async fn extract_user<'a>() -> ServFnResult<Option<User<'a>>> {
    let Some(user_session) = extract_user_session().await? else {
        return Ok(None);
    };

    Ok(get_user_by_id(user_session.user_id).await.ok())
}

#[cfg(feature = "server")]
async fn extract_user_session() -> ServFnResult<Option<UserSession>> {
    let session = extract_session().await?;

    Ok(session.user_session().await.ok())
}

#[server]
pub async fn get_all_folders(parent_folder_id: Option<Uuid>) -> ServFnResult<Vec<FolderPresenter>> {
    require_login().await?;

    let user = extract_user().await?.unwrap();
    let parent_folder = if let Some(id) = parent_folder_id {
        Some(
            get_folder_by_id(id, Some(&user))
                .await
                .map_err(|_| ServFnError::Other("Could not get parent folder".to_owned()))?,
        )
    } else {
        None
    };
    let folders = get_all_folders_by_user(&user, parent_folder.as_ref()).await;

    Ok(futures::future::join_all(folders.iter().map(|folder| folder.async_into())).await)
}

#[server]
pub async fn get_current_user() -> ServFnResult<Option<UserPresenter>> {
    let Some(user) = extract_user().await? else {
        return Ok(None);
    };

    Ok(Some(user.into()))
}

#[server]
pub async fn get_folder(id: Uuid) -> ServFnResult<Option<FolderPresenter>> {
    require_login().await?;

    let user = extract_user().await?.unwrap();

    let result = get_folder_by_id(id, Some(&user)).await;

    Ok(if let Ok(folder) = result {
        Some(folder.async_into().await)
    } else {
        None
    })
}

#[server]
pub async fn is_logged_in() -> ServFnResult<bool> {
    Ok(extract_user_session().await?.is_some())
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
