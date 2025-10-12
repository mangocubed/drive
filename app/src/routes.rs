use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use uuid::Uuid;

use crate::layouts::UserLayout;
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

    #[route("/authorized?:token&:expires_at")]
    AuthorizedPage { token: String, expires_at: DateTime<Utc> },
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
}
