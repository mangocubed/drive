use dioxus::prelude::*;

use crate::forms::FormStatus;
use crate::inputs::RegisterInput;

#[cfg(feature = "server")]
use crate::server::commands::insert_user;

#[server]
pub async fn attempt_to_register(input: RegisterInput) -> Result<FormStatus, ServerFnError> {
    let result = insert_user(&input).await;

    match result {
        Ok(_) => Ok(FormStatus::Success("User created successfully".to_owned())),
        Err(errors) => Ok(FormStatus::Failed("Failed to create user".to_owned(), errors)),
    }
}
