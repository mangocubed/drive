use dioxus::prelude::*;

use crate::components::ConfirmationModal;
use crate::icons::{Bars3Outline, ChevronDownMini};
use crate::routes::Routes;
use crate::server_functions::attempt_to_logout;
use crate::use_current_user;

const ICON_SVG: Asset = asset!("assets/icon.svg");

#[component]
pub fn NavbarItems() -> Element {
    rsx! {
        li {
            Link { to: Routes::home(), "Home" }
        }
    }
}

#[component]
pub fn Layout() -> Element {
    let mut is_loading = use_signal(|| true);
    let mut show_logout_confirmation = use_signal(|| false);
    let navigator = use_navigator();
    let mut current_user = use_current_user();

    use_effect(move || is_loading.set(false));

    rsx! {
        div { class: "navbar bg-base-300 shadow-md px-3",
            div { class: "navbar-start",
                div { class: "dropdown",
                    button { class: "btn btn-ghost lg:hidden", tabindex: 0, Bars3Outline {} }

                    ul {
                        class: "menu menu-sm dropdown-content bg-base-200 rounded-box shadow mt-3 p-2 w-52 z-1",
                        tabindex: 0,
                        NavbarItems {}
                    }
                }

                Link { class: "flex gap-2 items-center", to: Routes::home(),
                    img { class: "h-[36px]", src: ICON_SVG }
                    div { class: "text-xl font-bold", "Lime3" }
                }
            }

            div { class: "navbar-center hidden lg:flex",
                ul { class: "menu menu-horizontal px-1", NavbarItems {} }
            }

            div { class: "navbar-end",
                if let Some(Some(user)) = &*current_user.read() {
                    div { class: "dropdown dropdown-end",
                        button { class: "btn btn-ghost btn-lg px-2", tabindex: 1,
                            div { class: "text-left text-xs",
                                div { class: "mb-1 font-bold", {user.display_name.clone()} }
                                div { class: "opacity-70",
                                    "@"
                                    {user.username.clone()}
                                }
                            }
                            ChevronDownMini {}
                        }

                        ul {
                            class: "menu menu-sm dropdown-content bg-base-200 rounded-box shadow mt-3 p-2 w-48 z-1",
                            tabindex: 1,
                            li {
                                a {
                                    onclick: move |_| {
                                        *show_logout_confirmation.write() = true;
                                    },
                                    "Logout"
                                }
                            }
                        }
                    }

                    ConfirmationModal {
                        is_open: show_logout_confirmation,
                        on_accept: move |()| {
                            async move {
                                if attempt_to_logout().await.is_ok() {
                                    navigator.push(Routes::login());
                                    current_user.restart();
                                }
                            }
                        },
                        "Are you sure you want to logout?"
                    }
                } else {
                    Link { class: "btn", to: Routes::login(), "Login" }
                }
            }
        }

        main { class: "main", Outlet::<Routes> {} }

        div { class: "loading-overlay", class: if !is_loading() { "is-done" },
            figure {
                div { class: "loading-pulse" }
                img { src: ICON_SVG }
            }
        }
    }
}
