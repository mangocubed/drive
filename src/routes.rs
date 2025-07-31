use dioxus::prelude::*;

use crate::layout::Layout;
use crate::pages::{HomePage, LoginPage, RegisterPage};

#[derive(Clone, Routable)]
#[rustfmt::skip]
#[allow(clippy::enum_variant_names)]
pub enum Routes {
    #[layout(Layout)]
        #[route("/")]
        HomePage {},
        #[route("/login")]
        LoginPage {},
        #[route("/register")]
        RegisterPage {},
}

impl Routes {
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
