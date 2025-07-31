use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use tower_sessions::Session;
#[cfg(feature = "server")]
use validator::ValidationErrors;

use crate::forms::FormStatus;
use crate::inputs::{LoginInput, RegisterInput};
use crate::presenters::UserPresenter;
use crate::routes::Routes;

#[cfg(feature = "server")]
use crate::server::commands::{
    authenticate_user, delete_user_session, get_user_by_id, insert_user, insert_user_session,
};
#[cfg(feature = "server")]
use crate::server::models::{User, UserSession};
#[cfg(feature = "server")]
use crate::server::session::SessionTrait;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServFnError {
    LoginRequired,
    NoLoginRequired,
    Other(String),
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
        .map_err(|_| ServerFnError::ServerError(ServFnError::Other("Session layer not found".to_owned())))
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
pub async fn get_current_user() -> ServFnResult<Option<UserPresenter>> {
    let Some(user) = extract_user().await? else {
        return Ok(None);
    };

    Ok(Some(user.into()))
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
