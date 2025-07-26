use daisy_rsx::Input;
use dioxus::prelude::*;

#[component]
pub fn Form(children: Element) -> Element {
    rsx! {
        form { class: "form", autocomplete: "off", novalidate: "true", {children} }
    }
}

#[component]
pub fn FormField(children: Element, error: ReadSignal<Option<String>>, id: String, label: String) -> Element {
    rsx! {
        fieldset { class: "fieldset",
            label { class: "fieldset-label empty:hidden", r#for: id, {label} }

            {children}

            div { class: "fieldset-label text-error empty:hidden", {error} }
        }
    }
}

#[component]
pub fn PasswordField(
    id: String,
    label: String,
    name: String,
) -> Element {
    let mut input_type = use_signal(|| "password");
    
    rsx! {
        FormField {
            error, id, label,
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
                    onclick: |event| {
                        event.prevent_default();
                        
                        input_type = if value == "password" {
                            "text"
                        } else {
                            "password"
                        };
                    }
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
    rsx! {
        FormField {
            error, id, label,
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
