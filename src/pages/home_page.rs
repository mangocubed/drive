use dioxus::prelude::*;

use crate::components::{FolderManager, PageTitle};

#[component]
pub fn HomePage() -> Element {
    rsx! {
        PageTitle { "Home" }

        h1 { class: "h1", "Home" }

        FolderManager {}
    }
}
