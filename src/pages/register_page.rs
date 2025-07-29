use dioxus::prelude::*;

use crate::components::PageTitle;
use crate::forms::{Form, PasswordField, SelectField, TextField, use_form_provider};
use crate::server_functions::attempt_to_register;

#[component]
pub fn RegisterPage() -> Element {
    let form_provider = use_form_provider(attempt_to_register);

    rsx! {
        PageTitle { "Register" }

        h1 { class: "h1", "Register" }

        Form { provider: form_provider,
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
    }
}
