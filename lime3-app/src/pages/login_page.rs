use dioxus::prelude::*;
use serde_json::Value;

use crate::components::PageTitle;
use crate::forms::{Form, FormSuccessModal, PasswordField, TextField, use_form_provider};
use crate::routes::Routes;
use crate::server_fns::attempt_to_login;
use crate::use_current_user;
use crate::utils::{DataStorageTrait, data_storage};

#[component]
pub fn LoginPage() -> Element {
    use_form_provider(attempt_to_login);

    let navigator = use_navigator();
    let mut current_user = use_current_user();

    rsx! {
        PageTitle { "Login" }

        h1 { class: "h1", "Login" }

        FormSuccessModal {
            on_close: move |_| {
                navigator.push(Routes::home());
                current_user.restart();
            },
        }

        Form {
            on_success: move |value| {
                if let Value::String(token) = value {
                    data_storage().set_access_token(&token);
                }
            },
            TextField {
                id: "username_or_email",
                label: "Username or email",
                name: "username_or_email",
            }

            PasswordField {
                id: "password",
                label: "Password",
                max_length: 128,
                name: "password",
            }
        }

        div { class: "max-w-[640px] ml-auto mr-auto mt-4 flex flex-col gap-4",
            Link { class: "btn btn-block btn-outline", to: Routes::register(),
                "I don't have an account"
            }
        }
    }
}
