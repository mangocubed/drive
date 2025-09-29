use dioxus::prelude::*;

use sdk::components::PageTitle;

use crate::components::FileManager;

#[component]
pub fn HomePage() -> Element {
    rsx! {
        PageTitle { "Home" }

        h1 { class: "h2 breadcrumbs", "Home" }

        FileManager {}
    }
}
