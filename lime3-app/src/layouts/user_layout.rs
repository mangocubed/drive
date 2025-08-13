use dioxus::prelude::*;

use crate::icons::{ChevronDownMini, CloudOutline};
use crate::routes::Routes;
use crate::server_functions::attempt_to_logout;
use crate::use_current_user;
use crate::{components::ConfirmationModal, icons::HomeOutline};

use super::{ICON_SVG, LoadingOverlay};

#[component]
pub fn UserLayout() -> Element {
    let mut show_logout_confirmation = use_signal(|| false);
    let navigator = use_navigator();
    let mut current_user = use_current_user();

    use_effect(move || {
        if let Some(None) = *current_user.read() {
            navigator.push(Routes::login());
        }
    });

    rsx! {
        if let Some(Some(user)) = &*current_user.read() {
            div { class: "navbar bg-base-300 shadow-md px-3",
                div { class: "navbar-start",
                    Link { class: "flex gap-2 items-center", to: Routes::home(),
                        img { class: "h-[36px]", src: ICON_SVG }
                        div { class: "text-xl font-bold", "Lime3" }
                    }
                }

                div { class: "navbar-end",
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
                }
            }

            div { class: "flex m-4 min-h-[calc(100vh-6rem)]",
                div { class: "shrink-0 bg-base-200 rounded-box md:w-56 flex flex-col items-between",
                    ul { class: "menu md:w-56",
                        li {
                            class: "max-md:tooltip max-md:tooltip-right",
                            "data-tip": "Home",
                            Link { to: Routes::home(),
                                HomeOutline {}
                                span { class: "max-md:hidden", "Home" }
                            }
                        }
                    }

                    div { class: "mt-auto p-4 max-md:tooltip max-md:tooltip-right",
                        div { class: "flex gap-2 text-sm items-center max-md:justify-center",
                            CloudOutline {}
                            span { class: "max-md:hidden", "Storage" }
                        }

                        div { class: "tooltip-content max-md:w-48",
                            div { class: "md:hidden text-left", "Storage" }
                            progress {
                                class: "progress progress-primary w-full",
                                value: user.used_storage_bytes,
                                max: user.total_storage_bytes,
                            }
                            div { class: "text-xs text-right",
                                {user.used_storage.clone()}
                                " of "
                                {user.total_storage.clone()}
                                " used"
                            }
                        }
                    }
                }

                main { class: "main grow", Outlet::<Routes> {} }
            }
        }

        LoadingOverlay {}
    }
}
