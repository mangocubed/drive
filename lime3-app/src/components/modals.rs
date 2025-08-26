use dioxus::prelude::*;

use crate::server_functions::{attempt_to_update_space, get_pricing};
use crate::use_current_user;

#[component]
pub fn ConfirmationModal(children: Element, is_open: Signal<bool>, on_accept: Callback) -> Element {
    rsx! {
        Modal { is_closable: false, is_open,
            div { {children} }

            div { class: "modal-action",
                button {
                    class: "btn",
                    onclick: move |event| {
                        event.prevent_default();
                        *is_open.write() = false;
                    },
                    "Cancel"
                }
                button {
                    class: "btn btn-primary",
                    onclick: move |event| {
                        event.prevent_default();
                        *is_open.write() = false;
                        on_accept.call(());
                    },
                    "Accept"
                }
            }
        }
    }
}

#[component]
pub fn EditSpaceModal(is_open: Signal<bool>, on_success: Callback<()>) -> Element {
    let current_user = use_current_user();
    let pricing = use_resource(get_pricing);
    let mut value_space_gib = use_signal(|| 0);

    use_effect(move || {
        if let Some(Some(user)) = &*current_user.read() {
            *value_space_gib.write() = user.total_space_gib;
        }
    });

    rsx! {
        Modal { class: "max-w-300", is_open,
            h2 { class: "h2", "Edit space" }

            div { class: "flex gap-2 mb-4",
                div { class: "grow",
                    if let Some(Ok(pricing)) = &*pricing.read() {
                        input {
                            class: "range w-full mb-1",
                            min: pricing.free_quota_gib,
                            max: pricing.max_quota_gib,
                            oninput: move |event| {
                                *value_space_gib.write() = event.value().parse().unwrap();
                            },
                            step: 1,
                            r#type: "range",
                            value: value_space_gib,
                        }

                        div { class: "flex justify-between",
                            span {
                                {pricing.free_quota_gib.to_string()}
                                " GiB"
                            }

                            span {
                                {pricing.max_quota_gib.to_string()}
                                " GiB"
                            }
                        }
                    }
                }

                span { class: "font-bold",
                    {value_space_gib().to_string()}
                    " GiB"
                }
            }


            button {
                class: "btn btn-primary btn-block",
                onclick: move |event| {

                    event.prevent_default();
                    async move {
                        let _ = attempt_to_update_space(value_space_gib()).await;
                        *is_open.write() = false;
                        on_success.call(());
                    }
                },
                "Submit"
            }
        }
    }
}

#[component]
pub fn Modal(
    children: Element,
    #[props(optional)] class: String,
    is_open: Signal<bool>,
    #[props(default = true)] is_closable: bool,
    #[props(optional)] on_close: Callback<MouseEvent>,
) -> Element {
    let on_close = move |event: MouseEvent| {
        event.prevent_default();
        *is_open.write() = false;
        on_close.call(event);
    };

    rsx! {
        dialog { class: "modal", class: if is_open() { "modal-open" },
            if is_closable {
                button {
                    class: "btn btn-sm btn-circle btn-ghost absolute right-2 top-2",
                    onclick: on_close,
                    "✕"
                }
            }

            div { class: format!("modal-box {class}"), {children} }

            if is_closable {
                div { class: "modal-backdrop", onclick: on_close }
            }
        }
    }
}
