use chrono::{DateTime, Utc};
use dioxus::prelude::*;

use sdk::components::PageTitle;
use sdk::hooks::use_resource_with_loader;

use crate::hooks::use_current_user;
use crate::local_data::{delete_redirect_to, get_redirect_to, set_session_token};
use crate::routes::Routes;
use crate::server_fns::attempt_to_confirm_authorization;

#[component]
pub fn AuthorizedPage(token: String, expires_at: DateTime<Utc>) -> Element {
    let mut current_user = use_current_user();
    let navigator = use_navigator();
    let confirm_authorization = use_resource_with_loader("confirm-authorization", move || {
        attempt_to_confirm_authorization(token.clone(), expires_at)
    });

    use_effect(move || {
        if let Some(Some(_)) = &*current_user.read() {
            navigator.push(get_redirect_to());
            delete_redirect_to();
        }
    });

    rsx! {
        PageTitle { "Confirm authorization" }

        main { class: "main",
            div { class: "text-center",
                match confirm_authorization() {
                    Some(Ok(session_token)) => {
                        set_session_token(&session_token);
                        current_user.restart();

                        rsx! {
                            div { class: "text-lg font-bold", "Authorization confirmed successfully" }
                            Link { class: "btn btn-link", to: Routes::home(), "Go to home" }
                        }
                    }
                    Some(_) => rsx! {
                        div { class: "text-lg font-bold",
                            "Failed to confirm checkout. Please contact the support if you need help."
                        }
                    },
                    _ => VNode::empty(),
                }
            }
        }

    }
}
