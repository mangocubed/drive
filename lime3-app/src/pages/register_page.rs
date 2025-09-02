use dioxus::prelude::*;
use serde_json::Value;

use crate::components::PageTitle;
use crate::forms::{Form, FormSuccessModal, PasswordField, SelectField, TextField, use_form_provider};
use crate::routes::Routes;
use crate::server_fns::attempt_to_register;
use crate::use_current_user;
use crate::utils::{DataStorageTrait, data_storage};

#[component]
pub fn RegisterPage() -> Element {
    use_form_provider(attempt_to_register);

    let navigator = use_navigator();
    let mut current_user = use_current_user();

    rsx! {
        PageTitle { "Register" }

        h1 { class: "h1", "Register" }

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
                id: "username",
                label: "Username",
                max_length: 16,
                name: "username",
            }

            TextField {
                id: "email",
                input_type: "email",
                label: "Email",
                name: "email",
            }

            PasswordField {
                id: "password",
                label: "Password",
                max_length: 128,
                name: "password",
            }

            TextField { id: "full_name", label: "Full name", name: "full_name" }

            TextField {
                id: "birthdate",
                label: "Birthdate",
                name: "birthdate",
                input_type: "date",
            }

            SelectField {
                id: "country_alpha2",
                label: "Country",
                name: "country_alpha2",
                option { "Select" }
                for country in rust_iso3166::ALL {
                    option { value: country.alpha2, {country.name} }
                }
            }
        }

        div { class: "max-w-[640px] ml-auto mr-auto mt-4 flex flex-col gap-4",
            Link { class: "btn btn-block btn-outline", to: Routes::login(), "Back to login" }
        }
    }
}
