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
        #[route("/trash")]
        TrashPage {},
    #[end_layout]

    #[layout(GuestLayout)]
        #[route("/login")]
        LoginPage {},
        #[route("/register")]
        RegisterPage {},
    #[end_layout]

    #[route("/confirm-checkout?:checkout_id")]
    ConfirmCheckoutPage { checkout_id: Uuid },
}

impl Routes {
    pub fn home() -> Self {
        Self::HomePage {}
    }

    pub fn file(id: Uuid) -> Self {
        Self::FilePage { id }
    }

    pub fn folder(id: Uuid) -> Self {
        Self::FolderPage { id }
    }

    pub fn storage() -> Self {
        Self::StoragePage {}
    }

    pub fn trash() -> Self {
        Self::TrashPage {}
    }

    pub fn login() -> Self {
        Self::LoginPage {}
    }

    pub fn register() -> Self {
        Self::RegisterPage {}
    }
}
