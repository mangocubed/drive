use dioxus::prelude::*;

use crate::components::NavbarBrand;
use crate::routes::Routes;
use crate::server_fns::is_logged_in;
use crate::use_resource_with_loader;

#[component]
pub fn GuestLayout() -> Element {
    let is_logged_in = use_resource_with_loader("logged-in".to_owned(), is_logged_in);
    let navigator = use_navigator();

    use_effect(move || {
        if let Some(Ok(true)) = is_logged_in() {
            navigator.push(Routes::home());
        }
    });

    rsx! {
        div { class: "navbar bg-base-300 shadow-md px-3",
            div { class: "navbar-start", NavbarBrand {} }
        }

        main { class: "main", Outlet::<Routes> {} }
    }
}
