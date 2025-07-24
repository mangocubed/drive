use dioxus::prelude::*;

use crate::routes::Routes;

#[component]
pub fn Layout() -> Element {
    rsx! {
        main {
            class: "main",
            Outlet::<Routes> {}
        }
    }
}
