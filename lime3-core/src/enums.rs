use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "server", derive(sqlx::Type))]
#[cfg_attr(feature = "server", sqlx(type_name = "file_visibility"))]
#[cfg_attr(feature = "server", sqlx(rename_all = "lowercase"))]
#[serde(rename_all = "lowercase")]
pub enum FileVisibility {
    Private,
    Followers,
    Users,
    Public,
}

impl Display for FileVisibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileVisibility::Private => write!(f, "private"),
            FileVisibility::Followers => write!(f, "followers"),
            FileVisibility::Users => write!(f, "users"),
            FileVisibility::Public => write!(f, "public"),
        }
    }
}
