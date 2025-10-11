use dioxus::prelude::*;
use uuid::Uuid;

use sdk::components::PageTitle;
use sdk::hooks::use_resource_with_loader;

use crate::hooks::use_current_user;
use crate::routes::Routes;
use crate::server_fns::attempt_to_confirm_plan_checkout;

#[component]
pub fn ConfirmCheckoutPage(checkout_id: Uuid) -> Element {
    let mut current_user = use_current_user();
    let confirm_checkout = use_resource_with_loader("confirm-checkout", move || {
        attempt_to_confirm_plan_checkout(checkout_id)
    });

    use_effect(move || {
        if let Some(Ok(_)) = confirm_checkout() {
            current_user.restart();
        }
    });

    rsx! {
        PageTitle { "Confirm checkout" }

        main { class: "main",
            div { class: "text-center",
                match confirm_checkout() {
                    Some(Ok(message)) => rsx! {
                        div { class: "text-lg font-bold", {message} }
                        Link { class: "btn btn-link", to: Routes::home(), "Go to home" }
                    },
                    Some(_) => rsx! {
                        div { class: "text-lg font-bold",
                            "Failed to confirm checkout. Please contact the support if you need help."
                        }
                    },
                    _ => rsx! {},
                }
            }
        }
    }
}
