use dioxus::prelude::*;

use crate::icons::Bars3Outline;
use crate::routes::Routes;

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

    use_effect(move || is_loading.set(false));

    rsx! {
        div { class: "navbar bg-base-300 shadow-md px-3",
            div { class: "navbar-start",
                div { class: "dropdown",
                    button { class: "btn btn-ghost lg:hidden", Bars3Outline {} }

                    ul { class: "menu menu-sm dropdown-content bg-base-200 rounded-box z-1 mt-3 w-52 p-2 shadow",
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
                Link { class: "btn", to: Routes::register(), "Register" }
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
