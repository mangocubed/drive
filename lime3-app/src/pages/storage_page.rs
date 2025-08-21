use dioxus::prelude::*;

use crate::components::{MembershipsModal, PageTitle};
use crate::use_current_user;

#[component]
pub fn StoragePage() -> Element {
    let mut current_user = use_current_user();
    let mut show_memberships_modal = use_signal(|| false);

    rsx! {
        PageTitle { "Storage" }

        h1 { class: "h1", "Storage" }

        if let Some(Some(user)) = &*current_user.read() {
            progress {
                class: "progress progress-primary w-full h-4",
                value: user.used_storage_bytes,
                max: user.total_storage_bytes,
            }
            div { class: "text-xs text-right",
                {user.used_storage.clone()}
                " of "
                {user.total_storage.clone()}
                " used"
            }

            h2 { class: "h2", "Current membership" }

            div { class: "card card-border border-3 mx-auto",
                div { class: "card-body",
                    h3 { class: "h3", {user.membership.name.clone()} }

                    div { class: "mb-1 text-lg",
                        if user.membership.is_free {
                            "Free"
                        } else if user.membership_is_annual {
                            {user.membership.annual_price.clone()}
                            " / year"
                        } else {
                            {user.membership.monthly_price.clone()}
                            " / month"
                        }
                    }

                    div { class: "mb-1", {user.membership.description.clone()} }

                    ul { class: "mb-1 list",
                        li { class: "list-row",
                            {user.membership.total_storage.clone()}
                            " storage"
                        }
                        li { class: "list-row",
                            {user.membership.max_size_per_file.clone()}
                            " per file"
                        }
                    }

                    button {
                        class: "btn btn-primary w-full mt-auto",
                        onclick: move |_| {
                            *show_memberships_modal.write() = true;
                        },
                        if user.membership.is_free {
                            "Upgrade"
                        } else {
                            "Change"
                        }
                    }
                }
            }

            MembershipsModal {
                is_open: show_memberships_modal,
                on_success: move |_| {
                    current_user.restart();
                },
            }
        }
    }
}
