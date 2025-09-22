use dioxus::prelude::*;
use serde_json::Value;

use crate::components::Modal;
use crate::hooks::{FormStatus, use_form_context};
use crate::icons::{EyeMini, EyeSlashMini};

fn on_keydown(event: KeyboardEvent) {
    if event.key() == Key::Enter {
        event.prevent_default();
    }
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

#[component]
pub fn Form(children: Element, #[props(optional)] on_success: Callback<Value>) -> Element {
    let form_context = use_form_context();

    use_effect(move || {
        if let FormStatus::Success(_, data) = &*form_context.status.read() {
            on_success.call(data.clone())
        }
    });

    rsx! {
        form {
            class: "form",
            autocomplete: false,
            novalidate: true,
            onsubmit: move |event| {
                event.prevent_default();

                form_context.callback.call(event.data().values());
            },
            if let FormStatus::Failed(message, _) = &*form_context.status.read() {
                div { class: "py-2 has-[div:empty]:hidden",
                    div { class: "alert alert-error", role: "alert", {message.clone()} }
                }
            }

            {children}

            div { class: "py-3 w-full",
                button {
                    class: "btn btn-block btn-primary",
                    onclick: move |event| {
                        if form_context.is_pending() {
                            event.prevent_default();
                        }
                    },
                    r#type: "submit",
                    if form_context.is_pending() {
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
pub fn FormField(children: Element, error: Memo<Option<String>>, label: String) -> Element {
    rsx! {
        fieldset { class: "fieldset",
            legend { class: "fieldset-legend empty:hidden", {label} }

            {children}

            div { class: "label text-error empty:hidden", {error} }
        }
    }
}

#[component]
pub fn FormSuccessModal(#[props(optional)] on_close: Callback<Value>) -> Element {
    let form_context = use_form_context();
    let mut is_open = use_signal(|| false);
    let mut success = use_signal(|| (String::new(), Value::Null));

    use_effect(move || {
        if let FormStatus::Success(message, data) = &*form_context.status.read() {
            *is_open.write() = true;
            *success.write() = (message.clone(), data.clone());
        }
    });

    rsx! {
        Modal { is_open, is_closable: false,
            {success().0}

            div { class: "modal-action",
                button {
                    class: "btn btn-primary",
                    onclick: {
                        move |event| {
                            event.prevent_default();
                            on_close.call(success().1);
                            *is_open.write() = false;
                        }
                    },
                    "Ok"
                }
            }
        }
    }
}

#[component]
pub fn PasswordField(id: String, label: String, #[props(default = 256)] max_length: u16, name: String) -> Element {
    let error = use_error_memo(id.clone());
    let mut input_type = use_signal(|| "password");

    rsx! {
        FormField { error, label,
            div {
                class: "input flex items-center gap-2 pr-0",
                class: if error().is_some() { "input-error" },
                input {
                    class: "grow",
                    id,
                    maxlength: max_length,
                    name,
                    onkeydown: on_keydown,
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
        FormField { error, label,
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
pub fn TextField(
    id: String,
    #[props(default = "text".to_owned())] input_type: String,
    label: String,
    #[props(default = 256)] max_length: u16,
    name: String,
    #[props(into, optional)] value: Signal<String>,
) -> Element {
    let error = use_error_memo(id.clone());

    rsx! {
        FormField { error, label,
            input {
                class: "input",
                class: if error().is_some() { "input-error" },
                id,
                maxlength: max_length,
                name,
                onkeydown: on_keydown,
                oninput: move |event| {
                    *value.write() = event.value();
                },
                r#type: input_type,
                value,
            }
        }
    }
}
