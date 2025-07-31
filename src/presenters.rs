use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "server")]
use crate::server::models::User;

#[derive(Deserialize, Serialize)]
pub struct UserPresenter {
    id: Uuid,
    pub username: String,
    pub display_name: String,
    pub initials: String,
}

#[cfg(feature = "server")]
impl From<User<'_>> for UserPresenter {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username.to_string(),
            display_name: user.display_name.to_string(),
            initials: user.initials(),
        }
    }
}
