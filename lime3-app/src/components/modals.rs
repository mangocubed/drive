use dioxus::prelude::*;

use crate::server_functions::{attempt_to_create_plan_checkout, get_all_available_plans};

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
pub fn SubscriptionModal(is_open: Signal<bool>, on_success: Callback<()>) -> Element {
    let navigator = use_navigator();
    let plans = use_resource(get_all_available_plans);
    let mut selected_plan_id = use_signal(|| None);
    let mut is_yearly = use_signal(|| false);

    rsx! {
        Modal { class: "max-w-300", is_open,
            h2 { class: "h2", "Select plan" }

            div { class: "mb-4 font-bold flex justify-center gap-2",
                a {
                    onclick: move |event| {
                        event.prevent_default();
                        *is_yearly.write() = false;
                    },
                    "Monthly"
                }
                input {
                    checked: is_yearly,
                    class: "toggle",
                    oninput: move |event| {
                        *is_yearly.write() = event.checked();
                    },
                    r#type: "checkbox",
                }
                a {
                    onclick: move |event| {
                        event.prevent_default();
                        *is_yearly.write() = true;
                    },
                    "Yearly"
                }
            }

            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2",
                if let Some(Ok(plans)) = &*plans.read() {
                    for plan in plans {
                        div { class: "card card-border card-sm mb-4",
                            div { class: "card-body",
                                h3 { class: "card-title text-xl", {plan.name.clone()} }

                                p { {plan.description.clone()} }

                                p {
                                    if is_yearly() {
                                        span { class: "font-bold text-lg", {plan.yearly_price.clone()} }
                                        " per year"
                                    } else {
                                        span { class: "font-bold text-lg",
                                            {plan.monthly_price.clone()}
                                        }
                                        " per month"
                                    }
                                }

                                p {
                                    span { class: "font-bold text-lg", {plan.quota.clone()} }
                                    " of space"
                                }

                                button {
                                    class: "btn btn-block",
                                    disabled: selected_plan_id().is_some() && selected_plan_id() != Some(plan.id),
                                    onclick: {
                                        let plan_id = plan.id;
                                        move |event| {
                                            event.prevent_default();

                                            async move {
                                                if selected_plan_id().is_some() {
                                                    return;
                                                }

                                                *selected_plan_id.write() = Some(plan_id);

                                                let result = attempt_to_create_plan_checkout(plan_id, is_yearly()).await;

                                                if let Ok(checkout_url) = result {
                                                    navigator.push(checkout_url.to_string());
                                                } else {
                                                    *selected_plan_id.write() = None;
                                                }

                                            }
                                        }
                                    },
                                    if selected_plan_id() == Some(plan.id) {
                                        span { class: "loading loading-spinner" }
                                    } else {
                                        "Select"
                                    }
                                }
                            }
                        }
                    }
                }
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
