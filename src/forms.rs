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
pub fn TextField(
    error: ReadSignal<Option<String>>,
    id: String,
    #[props(default = "text".to_owned())] input_type: String,
    label: String,
) -> Element {
    rsx! {
        FormField { error, id, label,
            input {
                class: "input",
                class: if error().is_some() { "input-error" },
                id,
                r#type: input_type,
            }
        }
    }
}
