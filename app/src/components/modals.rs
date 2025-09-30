use dioxus::prelude::*;

use sdk::components::{Brand, Modal};

use crate::constants::{COPYRIGHT, PRIVACY_URL, SOURCE_CODE_URL, TERMS_URL};
use crate::forms::{Form, TextField};
use crate::hooks::use_form_provider;
use crate::presenters::FolderItemPresenter;
use crate::server_fns::{
    attempt_to_create_plan_checkout, attempt_to_rename_file, attempt_to_rename_folder, get_all_available_plans,
};
use crate::use_resource_with_loader;
use crate::utils::run_with_loader;

#[component]
pub fn AboutModal(is_open: Signal<bool>) -> Element {
    rsx! {
        Modal { is_open, class: "gap-4 flex flex-col items-center",
            Brand { "Drive" }

            div { class: "text-center text-sm opacity-75",
                p {
                    "Version: "
                    {env!("CARGO_PKG_VERSION")}
                    " ("
                    {env!("GIT_REV_SHORT")}
                    ")"
                }

                p {
                    "Built on: "
                    {env!("BUILD_DATETIME")}
                }
            }

            div {
                a { class: "link", href: TERMS_URL, target: "_blank", "Terms of Service" }

                span { class: "opacity-50", " | " }

                a { class: "link", href: PRIVACY_URL, target: "_blank", "Privacy Policy" }

                span { class: "opacity-50", " | " }

                a {
                    class: "link",
                    href: SOURCE_CODE_URL.clone(),
                    target: "_blank",
                    "Source code"
                }
            }

            div { class: "opacity-75", {COPYRIGHT} }
        }
    }
}

#[component]
pub fn RenameModal(
    is_open: Signal<bool>,
    #[props(into)] folder_item: FolderItemPresenter,
    on_success: Callback,
) -> Element {
    let mut form_provider = use_form_provider("rename".to_owned(), move |input| async move {
        if folder_item.is_file {
            attempt_to_rename_file(input).await
        } else {
            attempt_to_rename_folder(input).await
        }
    });

    let mut name_value = use_signal(|| folder_item.name.clone());

    use_effect(move || {
        if *is_open.read() {
            form_provider.reset();
            *name_value.write() = folder_item.name.clone();
        }
    });

    rsx! {
        Modal { is_open,
            h2 { class: "h2",
                "Rename "
                if folder_item.is_file {
                    "file"
                } else {
                    "folder"
                }
            }

            Form {
                on_success: move |_| {
                    on_success.call(());
                    *is_open.write() = false;
                },
                input {
                    name: "id",
                    value: folder_item.id.to_string(),
                    r#type: "hidden",
                }


                TextField {
                    id: "name",
                    label: "Name",
                    name: "name",
                    value: name_value,
                }
            }
        }
    }
}

#[component]
pub fn SubscriptionModal(is_open: Signal<bool>, on_success: Callback<()>) -> Element {
    let plans = use_resource_with_loader("available-plans".to_owned(), get_all_available_plans);
    let mut selected_plan_id = use_signal(|| None);
    let mut is_yearly = use_signal(|| false);

    #[cfg(feature = "web")]
    let navigator = use_navigator();

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

                                                let result = run_with_loader(

                                                        "create-plan-checkout".to_owned(),
                                                        move || attempt_to_create_plan_checkout(plan_id, is_yearly()),
                                                    )
                                                    .await;
                                                #[allow(unused_variables)]
                                                if let Ok(checkout_url) = result {
                                                    #[cfg(feature = "web")] navigator.push(checkout_url.to_string());
                                                    #[cfg(feature = "desktop")]
                                                    let _ = dioxus::desktop::use_window()
                                                        .webview
                                                        .load_url(checkout_url.as_ref());
                                                    #[cfg(feature = "mobile")]
                                                    let _ = dioxus::mobile::use_window()
                                                        .webview
                                                        .load_url(checkout_url.as_ref());
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
