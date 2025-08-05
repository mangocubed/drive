use dioxus::prelude::*;

use crate::components::{PageTitle, RequireNoLogin};
use crate::forms::{Form, FormSuccessModal, PasswordField, TextField, use_form_provider};
use crate::routes::Routes;
use crate::server_functions::attempt_to_login;
use crate::use_current_user;

#[component]
pub fn LoginPage() -> Element {
    use_form_provider(attempt_to_login);

    let navigator = use_navigator();
    let mut current_user = use_current_user();

    rsx! {
        RequireNoLogin {
            PageTitle { "Login" }

            h1 { class: "h1", "Login" }

            FormSuccessModal {
                on_close: move |()| {
                    navigator.push(Routes::home());
                    current_user.restart();
                },
            }

            Form {
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
                Link {
                    class: "btn btn-block btn-outline",
                    to: Routes::register(),
                    "I don't have an account"
                }
            }
        }
    }
}
