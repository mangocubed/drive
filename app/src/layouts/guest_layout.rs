use dioxus::prelude::*;

use crate::components::Brand;
use crate::constants::{COPYRIGHT, PRIVACY_URL, SOURCE_CODE_URL, TERMS_URL};
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
        div { class: "flex flex-col min-h-screen",
            div { class: "navbar bg-base-300 shadow-md px-3",
                div { class: "navbar-start",
                    Link { class: "flex gap-2 items-center", to: Routes::login(), Brand {} }
                }
            }

            main { class: "main grow", Outlet::<Routes> {} }

            footer { class: "footer md:footer-horizontal bg-base-200 p-10",

                aside { class: "opacity-75",
                    p {
                        "Version: "
                        {env!("CARGO_PKG_VERSION")}
                        " ("
                        {env!("GIT_REV_SHORT")}
                        ")"
                    }

                    p {
                        "Built on: "
                        {env!("BUILD_DATETIME")}
                    }

                    p { {COPYRIGHT} }
                }

                nav {
                    a { class: "link", href: TERMS_URL, target: "_blank", "Terms of Service" }

                    a { class: "link", href: PRIVACY_URL, target: "_blank", "Privacy Policy" }

                    a {
                        class: "link",
                        href: SOURCE_CODE_URL.clone(),
                        target: "_blank",
                        "Source code"
                    }
                }
            }
        }
    }
}
