use dioxus::prelude::*;

use sdk::auth_client::{auth_client_authorize_url, auth_client_provider_url};
use sdk::components::{Brand, Navbar, NavbarEnd, NavbarStart};

use crate::components::AboutModal;
use crate::hooks::use_current_user;
use crate::icons::{ChevronDownMini, CloudOutline, HomeOutline, InformationCircleOutline, TrashOutline};
use crate::local_data::set_redirect_to;
use crate::routes::Routes;

#[component]
pub fn UserLayout() -> Element {
    let router = router();
    let mut show_about = use_signal(|| false);
    let current_user = use_current_user();

    use_effect(move || {
        if let Some(None) = *current_user.read() {
            set_redirect_to(&router.full_route_string());

            sdk::open_external_url(auth_client_authorize_url());
        }
    });

    rsx! {
        if let Some(Some(user)) = &*current_user.read() {
            Navbar {
                NavbarStart {
                    Link { to: Routes::home(),
                        Brand { "Drive" }
                    }
                }

                NavbarEnd {
                    div { class: "dropdown dropdown-end",
                        button { class: "btn btn-ghost btn-lg px-2", tabindex: 0,
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
                            class: "menu menu-sm dropdown-content bg-base-200 rounded-box shadow mt-3 p-2 w-max z-1",
                            tabindex: 0,
                            li {
                                a { href: auth_client_provider_url().to_string(),
                                    "My account"
                                }
                            }
                        }
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

                        div { class: "divider m-1" }

                        li {
                            class: "max-md:tooltip max-md:tooltip-right",
                            "data-tip": "Trash",
                            Link { to: Routes::trash(),
                                TrashOutline {}

                                span { class: "max-md:hidden", "Trash" }
                            }
                        }
                    }

                    ul { class: "menu md:w-56 mt-auto",
                        li {
                            Link {
                                class: "max-md:tooltip max-md:tooltip-right grid-cols-1 grid-rows-2 gap-0",
                                to: Routes::storage(),
                                div { class: "flex gap-2",
                                    CloudOutline {}
                                    span { class: "max-md:hidden", "Storage" }
                                }

                                div { class: "tooltip-content max-md:w-48",
                                    div { class: "md:hidden text-left", "Storage" }
                                    progress {
                                        class: "progress progress-primary w-full",
                                        value: user.used_space_bytes,
                                        max: user.total_space_bytes,
                                    }
                                    div { class: "text-xs text-right",
                                        {user.used_space.clone()}
                                        " of "
                                        {user.total_space.clone()}
                                        " used"
                                    }
                                }
                            }
                        }

                        div { class: "divider m-1" }

                        li {
                            class: "max-md:tooltip max-md:tooltip-right",
                            "data-tip": "About",
                            a {
                                onclick: move |_| {
                                    *show_about.write() = true;
                                },
                                InformationCircleOutline {}

                                span { class: "max-md:hidden", "About" }
                            }
                        }
                    }
                }

                main { class: "main grow max-w-[calc(100%-48px)] md:max-w-[calc(100%-208px)]",
                    Outlet::<Routes> {}
                }

                AboutModal { is_open: show_about }
            }
        }
    }
}
