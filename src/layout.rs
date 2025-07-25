use dioxus::prelude::*;

use crate::routes::Routes;

const ICON_SVG: Asset = asset!("assets/icon.svg");

#[component]
pub fn Layout() -> Element {
    let mut is_loading = use_signal(|| true);

    use_effect(move || is_loading.set(false));

    rsx! {
        main { class: "main", Outlet::<Routes> {} }

        div { class: "loading-overlay", class: if !is_loading() { "is-done" },
            figure {
                div { class: "loading-pulse" }
                img { src: ICON_SVG }
            }
        }
    }
}
