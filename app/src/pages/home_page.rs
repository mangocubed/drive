use dioxus::prelude::*;

use crate::components::{FileManager, PageTitle};

#[component]
pub fn HomePage() -> Element {
    rsx! {
        PageTitle { "Home" }

        h1 { class: "h2 breadcrumbs", "Home" }

        FileManager {}
    }
}
