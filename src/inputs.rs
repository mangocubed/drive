use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use validator::{Validate, ValidationError};

#[cfg(feature = "server")]
use crate::server::constants::REGEX_USERNAME;

#[cfg(feature = "server")]
fn validate_username(value: &String) -> Result<(), ValidationError> {
    use std::borrow::Cow;

    use uuid::Uuid;

    if Uuid::try_parse(value).is_ok() {
        return Err(ValidationError::new("invalid").with_message(Cow::Borrowed("Is invalid")));
    }

    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "server", derive(Validate))]
pub struct RegisterInput {
    #[cfg_attr(
        feature = "server",
        validate(
            length(min = 3, max = 16, message = "Must be between 3 and 16 characters"),
            regex(path = *REGEX_USERNAME, message = "Is invalid"),
            custom(function = "validate_username")
        )
    )]
    pub username: String,
    #[cfg_attr(
        feature = "server",
        validate(
            length(min = 5, max = 256, message = "Must be between 5 and 256 characters"),
            email(message = "Is invalid")
        )
    )]
    pub email: String,
    #[cfg_attr(
        feature = "server",
        validate(length(min = 6, max = 128, message = "Must be between 6 and 128 characters"))
    )]
    pub password: String,
    pub full_name: String,
    pub birthdate: NaiveDate,
    pub country_alpha2: String,
}
