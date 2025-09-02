use dioxus::prelude::*;

#[cfg(feature = "server")]
use serde_json::Value;
#[cfg(feature = "server")]
use validator::ValidationErrors;

use lime3_core::inputs::{LoginInput, RegisterInput};

#[cfg(feature = "server")]
use lime3_core::server::commands::{authenticate_user, delete_access_token, insert_access_token, insert_user};

use crate::forms::FormStatus;

use super::{ServFnClient, ServFnResult};

#[cfg(feature = "server")]
use super::{extract_access_token, require_login, require_no_login};

#[server(client = ServFnClient)]
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

    let result = insert_access_token(&user).await;

    match result {
        Ok(access_token) => Ok(FormStatus::Success(
            "User authenticated successfully".to_owned(),
            access_token.token.into(),
        )),
        Err(_) => Ok(FormStatus::Failed(
            "Failed to authenticate user".to_owned(),
            ValidationErrors::new(),
        )),
    }
}

#[server(client = ServFnClient)]
pub async fn attempt_to_logout() -> ServFnResult<()> {
    require_login().await?;

    let Some(access_token) = extract_access_token().await? else {
        return Ok(());
    };

    let _ = delete_access_token(&access_token).await;

    Ok(())
}

#[server(client = ServFnClient)]
pub async fn attempt_to_register(input: RegisterInput) -> ServFnResult<FormStatus> {
    require_no_login().await?;

    let result = insert_user(&input).await;

    match result {
        Ok(user) => {
            let result = insert_access_token(&user).await;

            let token = if let Ok(access_token) = result {
                access_token.token.into()
            } else {
                Value::Null
            };

            Ok(FormStatus::Success("User created successfully".to_owned(), token))
        }
        Err(errors) => Ok(FormStatus::Failed("Failed to create user".to_owned(), errors)),
    }
}
