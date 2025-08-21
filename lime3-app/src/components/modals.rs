use dioxus::prelude::*;

use crate::{
    presenters::MembershipPresenter,
    server_functions::{attempt_to_update_membership, get_available_memberships},
};

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
fn MembershipCard(
    is_annual: ReadOnlySignal<bool>,
    membership: MembershipPresenter,
    on_success: Callback<()>,
) -> Element {
    let navigator = use_navigator();

    rsx! {
        div { class: "card card-border",
            div { class: "card-body",
                h3 { class: "h3", {membership.name.clone()} }

                div { class: "mb-1",
                    if membership.is_free {
                        "Free"
                    } else if is_annual() {
                        span { class: "text-lg text-nowrap",
                            {membership.annual_price.clone()}
                            " / year"
                        }
                    } else {
                        span { class: "text-lg text-nowrap",
                            {membership.monthly_price.clone()}
                            " / month"
                        }
                    }
                }

                div { class: "mb-1", {membership.description.clone()} }

                ul { class: "mb-1 list",
                    li { class: "list-row",
                        {membership.total_storage.clone()}
                        " storage"
                    }
                    li { class: "list-row",
                        {membership.max_size_per_file.clone()}
                        " per file"
                    }
                }

                button { class: "btn btn-primary w-full mt-auto", onclick: move |event| {
                    event.prevent_default();

                    let membership_code = membership.code.clone();

                    async move {
                        let result = attempt_to_update_membership(membership_code, is_annual()).await;

                        match result {
                            Ok(Some(checkout_session_url)) => {
                                navigator.push(checkout_session_url.to_string());
                            }
                            Ok(None) => {
                                on_success.call(());
                            }
                            Err(_) => {
                            }
                        }
                    }
                }, "Select" }
            }
        }
    }
}

#[component]
pub fn MembershipsModal(is_open: Signal<bool>, on_success: Callback<()>) -> Element {
    let available_memberships = use_resource(get_available_memberships);
    let mut is_annual = use_signal(|| false);

    rsx! {
        Modal { class: "max-w-300", is_open,
           div { class: "mb-4 flex gap-2 justify-center",
               a { class: "cursor-pointer", onclick: move |event| { event.prevent_default(); *is_annual.write() = false; }, "Monthly" }

               input { checked: is_annual, class: "toggle", onchange: move |event| { *is_annual.write() = event.checked(); }, type: "checkbox" }

               a { class: "cursor-pointer", onclick: move |event| { event.prevent_default(); *is_annual.write() = true; }, "Annual" }
           }

            div { class: "grid gap-4 sm:grid-cols-2 lg:grid-cols-3",
                if let Some(Ok(memberships)) = &*available_memberships.read() {
                    for membership in memberships {
                        MembershipCard { is_annual, membership: membership.clone(), on_success: move |_| {
                            *is_annual.write() = false;
                            on_success.call(());
                        } }
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
