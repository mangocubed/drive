use dioxus::prelude::*;

use crate::forms::ActionResponse;
use crate::inputs::RegisterInput;

#[cfg(feature = "server")]
use crate::server::commands::insert_user;

#[server]
pub async fn attempt_to_register(input: RegisterInput) -> Result<ActionResponse, ServerFnError> {
    let result = insert_user(input).await;

    match result {
        Ok(user) => Ok(ActionResponse::Success("User created successfully".to_owned())),
        Err(errors) => Ok(ActionResponse::Error("Failed to create user".to_owned(), errors)),
    }
}
