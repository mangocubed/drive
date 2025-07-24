use dioxus::prelude::*;

use crate::layout::Layout;
use crate::pages::HomePage;

#[derive(Clone, Routable)]
pub enum Routes {
    #[layout(Layout)]
    #[route("/")]
    HomePage {},
}
