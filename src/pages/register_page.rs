use dioxus::prelude::*;
use validator::ValidationErrors;

use crate::components::PageTitle;
use crate::forms::{ActionResponse, Form, PasswordField, TextField};

#[component]
pub fn RegisterPage() -> Element {
    rsx! {
        PageTitle { "Register" }

        h1 { class: "h1", "Register" }

        Form {
            on_submit: move |event| {
                ActionResponse::Error(
                    "Failed to create user".to_owned(),
                    ValidationErrors::new(),
                )
            },

            TextField { id: "username", label: "Username", name: "username" }

            TextField {
                id: "email",
                input_type: "email",
                label: "Email",
                name: "email",
            }

            PasswordField { id: "password", label: "Password", name: "password" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_has_a_heading_text() {
        let element = dioxus_ssr::render_element(RegisterPage());
        let expected = dioxus_ssr::render_element(rsx! {
            h1 { class: "h1", "Register" }
        });

        pretty_assertions::assert_str_eq!(element, expected);
    }
}
