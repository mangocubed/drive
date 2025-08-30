use dioxus::prelude::*;
use uuid::Uuid;

use crate::components::PageTitle;
use crate::routes::Routes;
use crate::server_functions::attempt_to_confirm_plan_checkout;
use crate::use_current_user;

#[component]
pub fn ConfirmCheckoutPage(checkout_id: Uuid) -> Element {
    let mut current_user = use_current_user();
    let confirm_checkout = use_resource(move || attempt_to_confirm_plan_checkout(checkout_id));

    use_effect(move || {
        if let Some(Ok(_)) = confirm_checkout() {
            current_user.restart();
        }
    });

    rsx! {
        PageTitle { "Confirm checkout" }

        h1 { class: "h1", "Confirm checkout" }

        div { class: "text-center",
            if let Some(Ok(message)) = confirm_checkout() {
                div { class: "text-lg font-bold", {message} }

                Link { class: "btn btn-link", to: Routes::home(), "Go to home" }
            } else {
                div { class: "text-lg font-bold",
                    "Failed to confirm checkout. Please contact the support if you need help."
                }
            }
        }
    }
}
