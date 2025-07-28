use std::future::Future;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use validator::ValidationErrors;

use crate::components::Modal;

#[derive(Clone, Default, Deserialize, PartialEq, Serialize)]
pub enum ActionResponse {
    #[default]
    Nothing,
    Pending,
    Success(String),
    Error(String, ValidationErrors),
}

impl ActionResponse {
    fn is_pending(&self) -> bool {
        *self == ActionResponse::Pending
    }
}

fn use_error_memo(id: String) -> Memo<Option<String>> {
    let action_response = use_action_response();

    use_memo(move || {
        if let ActionResponse::Error(_, errors) = action_response.read().clone() {
            errors.field_errors().get(id.as_str()).and_then(|errors| {
                errors
                    .iter()
                    .find_map(|error| error.message.as_ref().map(|message| message.to_string()))
            })
        } else {
            None
        }
    })
}

fn use_action_response() -> Signal<ActionResponse> {
    use_context()
}

#[derive(Clone, PartialEq, Props)]
struct FormProps<T: Future<Output = Result<ActionResponse, ServerFnError>> + 'static> {
    children: Element,
    on_submit: Callback<Event<FormData>, T>,
}

#[component]
pub fn Form<T: Future<Output = Result<ActionResponse, ServerFnError>> + 'static>(props: FormProps<T>) -> Element {
    let mut action_response = use_signal(|| ActionResponse::default());

    use_context_provider(|| action_response);

    rsx! {
        form {
            class: "form",
            autocomplete: "off",
            novalidate: "true",
            onsubmit: move |event| {
                event.prevent_default();

                *action_response.write() = ActionResponse::Pending;

                async move {
                    if let Ok(response) = props.on_submit.call(event).await {
                        *action_response.write() = response;
                    }
                }
            },
            match action_response() {
                ActionResponse::Success(message) => rsx! {
                    SuccessModal { {message} }
                },
                ActionResponse::Error(message, _) => rsx! {
                    div { class: "py-2 has-[div:empty]:hidden",
                        div { class: "alert alert-error", role: "alert", {message} }
                    }
                },
                _ => rsx! {},
            }

            {props.children}

            SubmitButton {}
        }
    }
}

#[component]
pub fn FormField(children: Element, error: Memo<Option<String>>, id: String, label: String) -> Element {
    rsx! {
        fieldset { class: "fieldset",
            label { class: "fieldset-label empty:hidden", r#for: id, {label} }

            {children}

            div { class: "fieldset-label text-error empty:hidden", {error} }
        }
    }
}

#[component]
pub fn PasswordField(id: String, label: String, name: String) -> Element {
    let error = use_error_memo(id.clone());
    let mut input_type = use_signal(|| "password");

    rsx! {
        FormField { error, id: id.clone(), label,
            div {
                class: "input flex items-center gap-2 pr-0",
                class: if error().is_some() { "input-error" },
                input {
                    class: "grow",
                    id,
                    name,
                    r#type: input_type,
                }

                button {
                    class: "btn btn-ghost btn-sm",
                    onclick: move |event| {
                        event.prevent_default();

                        *input_type.write() = if input_type() == "password" {
                            "text"
                        } else {
                            "password"
                        };

                    },
                }
            }
        }
    }
}

#[component]
fn SubmitButton() -> Element {
    let action_response = use_action_response();

    rsx! {
        div { class: "py-3 w-full",
            button {
                class: "btn btn-block btn-primary",
                onclick: move |event| {
                    if action_response().is_pending() {
                        event.prevent_default();
                    }
                },
                r#type: "submit",
                if action_response().is_pending() {
                    span { class: "loading loading-spinner" }
                } else {
                    "Submit"
                }
            }
        }
    }
}

#[component]
fn SuccessModal(children: Element) -> Element {
    let is_open = use_signal(|| true);

    rsx! {
        Modal { is_open, is_closable: false,
            {children}
            div { class: "modal-action",
                button {
                    class: "btn btn-primary",
                    onclick: |event| {
                        event.prevent_default();
                    },
                    "Ok"
                }
            }
        }
    }
}

#[component]
pub fn TextField(
    id: String,
    #[props(default = "text".to_owned())] input_type: String,
    label: String,
    name: String,
) -> Element {
    let error = use_error_memo(id.clone());

    rsx! {
        FormField { error, id: id.clone(), label,
            input {
                class: "input",
                class: if error().is_some() { "input-error" },
                id,
                name,
                r#type: input_type,
            }
        }
    }
}
