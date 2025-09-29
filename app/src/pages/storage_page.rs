use dioxus::prelude::*;

use sdk::components::PageTitle;

use crate::components::SubscriptionModal;
use crate::hooks::use_current_user;

#[component]
pub fn StoragePage() -> Element {
    let mut current_user = use_current_user();
    let mut show_modal = use_signal(|| false);

    rsx! {
        PageTitle { "Storage" }

        h1 { class: "h1", "Storage" }

        if let Some(Some(user)) = &*current_user.read() {
            progress {
                class: "progress progress-primary w-full h-4",
                value: user.used_space_bytes,
                max: user.total_space_bytes,
            }
            div { class: "text-xs text-right",
                {user.used_space.clone()}
                " of "
                {user.total_space.clone()}
                " used"
            }

            if user.plan.is_none() {
                button {
                    class: "btn btn-primary w-full mt-4",
                    onclick: move |_| {
                        *show_modal.write() = true;
                    },
                    "Get more space"
                }

                SubscriptionModal {
                    is_open: show_modal,
                    on_success: move |_| {
                        current_user.restart();
                    },
                }
            }
        }
    }
}
