use dioxus::prelude::*;

use crate::layout::Layout;
use crate::pages::{HomePage, RegisterPage};

#[derive(Clone, Routable)]
pub enum Routes {
    #[layout(Layout)]
    #[route("/")]
    HomePage {},
    #[route("/register")]
    RegisterPage {},
}

impl Routes {
    pub fn home() -> Self {
        Routes::HomePage {}
    }

    pub fn register() -> Self {
        Routes::RegisterPage {}
    }
}
