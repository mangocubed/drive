use dioxus::prelude::*;
use uuid::Uuid;

use crate::layout::Layout;
use crate::pages::{FolderPage, HomePage, LoginPage, RegisterPage};

#[derive(Clone, Routable)]
#[rustfmt::skip]
#[allow(clippy::enum_variant_names)]
pub enum Routes {
    #[layout(Layout)]
        #[route("/folders/:id")]
        FolderPage {id: Uuid },
        #[route("/")]
        HomePage {},
        #[route("/login")]
        LoginPage {},
        #[route("/register")]
        RegisterPage {},
}

impl Routes {
    pub fn folder(id: Uuid) -> Self {
        Routes::FolderPage { id }
    }

    pub fn home() -> Self {
        Routes::HomePage {}
    }

    pub fn login() -> Self {
        Routes::LoginPage {}
    }

    pub fn register() -> Self {
        Routes::RegisterPage {}
    }
}
