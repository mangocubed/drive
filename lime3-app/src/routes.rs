use dioxus::prelude::*;
use uuid::Uuid;

use crate::layouts::{GuestLayout, UserLayout};
use crate::pages::*;

#[derive(Clone, Routable)]
#[rustfmt::skip]
#[allow(clippy::enum_variant_names)]
pub enum Routes {
    #[layout(UserLayout)]
        #[route("/")]
        HomePage {},
        #[route("/files/:id")]
        FilePage { id: Uuid },
        #[route("/folders/:id")]
        FolderPage { id: Uuid },
        #[route("/storage")]
        StoragePage {},
    #[end_layout]

    #[layout(GuestLayout)]
        #[route("/login")]
        LoginPage {},
        #[route("/register")]
        RegisterPage {},
}

impl Routes {
    pub fn home() -> Self {
        Routes::HomePage {}
    }

    pub fn file(id: Uuid) -> Self {
        Routes::FilePage { id }
    }

    pub fn folder(id: Uuid) -> Self {
        Routes::FolderPage { id }
    }

    pub fn storage() -> Self {
        Routes::StoragePage {}
    }

    pub fn login() -> Self {
        Routes::LoginPage {}
    }

    pub fn register() -> Self {
        Routes::RegisterPage {}
    }
}
