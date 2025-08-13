use dioxus::prelude::*;

mod guest_layout;
mod user_layout;

pub use guest_layout::GuestLayout;
pub use user_layout::UserLayout;

const ICON_SVG: Asset = asset!("assets/icon.svg");

#[component]
fn LoadingOverlay() -> Element {
    let mut is_loading = use_signal(|| true);

    use_effect(move || is_loading.set(false));

    rsx! {
        div { class: "loading-overlay", class: if !is_loading() { "is-done" },
            figure {
                div { class: "loading-pulse" }
                img { src: ICON_SVG }
            }
        }
    }
}
