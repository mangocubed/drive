use std::collections::HashMap;
use std::future::IntoFuture;

use dioxus::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::ValidationErrors;

use crate::components::Modal;
use crate::icons::{EyeMini, EyeSlashMini};

#[derive(Clone, PartialEq)]
pub struct FormProvider {
    action: Callback<HashMap<String, FormValue>>,
    status: ReadOnlySignal<FormStatus>,
}

impl FormProvider {
    fn is_pending(&self) -> bool {
        *self.status.read() == FormStatus::Pending
    }
}

#[derive(Clone, Default, Deserialize, PartialEq, Serialize)]
pub enum FormStatus {
    #[default]
    Nothing,
    Pending,
    Success(String),
    Failed(String, ValidationErrors),
}

fn use_error_memo(id: String) -> Memo<Option<String>> {
    let form_context = use_form_context();

    use_memo(move || {
        if let FormStatus::Failed(_, errors) = &*form_context.status.read() {
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

fn use_form_context() -> FormProvider {
    use_context()
}

pub fn use_form_provider<
    FA: Fn(I) -> R + Copy + 'static,
    I: Clone + DeserializeOwned + 'static,
    R: IntoFuture<Output = Result<FormStatus, ServerFnError>>,
>(
    action: FA,
) -> FormProvider {
    let mut status = use_signal(FormStatus::default);

    let action = use_callback(move |input: HashMap<String, FormValue>| {
        *status.write() = FormStatus::Pending;

        let input = serde_json::from_value(Value::Object(
            input
                .iter()
                .map(|(name, value)| (name.clone(), Value::String(value.as_value())))
                .collect(),
        ))
        .expect("Could not get input");

        spawn(async move {
            let result = action(input).await;

            if let Ok(response) = result {
                *status.write() = response;
            }
        });
    });

    use_context_provider(|| FormProvider {
        action,
        status: status.into(),
    })
}

#[component]
pub fn Form(children: Element, provider: FormProvider) -> Element {
    rsx! {
        form {
            class: "form",
            autocomplete: "off",
            novalidate: "true",
            onsubmit: move |event| {
                event.prevent_default();

                provider.action.call(event.data().values());
            },
            match &*provider.status.read() {
                FormStatus::Success(message) => rsx! {
                    SuccessModal { {message.clone()} }
                },
                FormStatus::Failed(message, _) => rsx! {
                    div { class: "py-2 has-[div:empty]:hidden",
                        div { class: "alert alert-error", role: "alert", {message.clone()} }
                    }
                },
                _ => rsx! {},
            }

            {children}

            div { class: "py-3 w-full",
                button {
                    class: "btn btn-block btn-primary",
                    onclick: move |event| {
                        if provider.is_pending() {
                            event.prevent_default();
                        }
                    },
                    r#type: "submit",
                    if provider.is_pending() {
                        span { class: "loading loading-spinner" }
                    } else {
                        "Submit"
                    }
                }
            }
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
pub fn PasswordField(id: String, label: String, #[props(default = 256)] max_length: u16, name: String) -> Element {
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
                    maxlength: max_length,
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
                    if input_type() == "password" {
                        EyeSlashMini {}
                    } else {
                        EyeMini {}
                    }
                }
            }
        }
    }
}

#[component]
pub fn SelectField(id: String, label: String, name: String, children: Element) -> Element {
    let error = use_error_memo(id.clone());

    rsx! {
        FormField { error, id: id.clone(), label,
            select {
                class: "select",
                class: if error().is_some() { "select-error" },
                id,
                name,
                {children}
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
    #[props(default = 256)] max_length: u16,
    name: String,
) -> Element {
    let error = use_error_memo(id.clone());

    rsx! {
        FormField { error, id: id.clone(), label,
            input {
                class: "input",
                class: if error().is_some() { "input-error" },
                id,
                maxlength: max_length,
                name,
                r#type: input_type,
            }
        }
    }
}
