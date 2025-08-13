use dioxus::prelude::*;

use crate::routes::Routes;
use crate::server_functions::is_logged_in;

use super::{ICON_SVG, LoadingOverlay};

#[component]
pub fn GuestLayout() -> Element {
    let is_logged_in = use_server_future(is_logged_in)?;
    let navigator = use_navigator();

    use_effect(move || {
        if let Some(Ok(true)) = is_logged_in() {
            navigator.push(Routes::home());
        }
    });

    rsx! {
        div { class: "navbar bg-base-300 shadow-md px-3",
            div { class: "navbar-start",
                Link { class: "flex gap-2 items-center", to: Routes::login(),
                    img { class: "h-[36px]", src: ICON_SVG }
                    div { class: "text-xl font-bold", "Lime3" }
                }
            }
        }

        main { class: "main", Outlet::<Routes> {} }

        LoadingOverlay {}
    }
}
