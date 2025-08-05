use dioxus::cli_config::app_title;
use dioxus::core::{DynamicNode, Template, TemplateNode};
use dioxus::prelude::*;
use uuid::Uuid;

use crate::enums::FileVisibility;
use crate::forms::{Form, FormSuccessModal, SelectField, TextField, use_form_provider};
use crate::icons::{FolderOutline, FolderPlusOutline};
use crate::routes::Routes;
use crate::server_functions::{attempt_to_create_folder, get_all_folders, is_logged_in};

const FILE_VISIBILITY_OPTIONS: [(&str, FileVisibility); 4] = [
    ("Private", FileVisibility::Private),
    ("Only followers", FileVisibility::Followers),
    ("Only users", FileVisibility::Users),
    ("Public", FileVisibility::Public),
];

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
pub fn FolderManager(
    #[props(default = FileVisibility::Private)] min_visibility: FileVisibility,
    parent_folder_id: Option<ReadOnlySignal<Uuid>>,
) -> Element {
    let mut show_new_folder_modal = use_signal(|| false);
    let mut all_folders = use_server_future(move || get_all_folders(parent_folder_id.map(|id| id())))?;

    rsx! {
        LoggedIn {
            button {
                class: "btn btn-outline",
                onclick: move |_| show_new_folder_modal.set(true),
                FolderPlusOutline {}
                "New folder"
            }

            NewFolderModal {
                is_open: show_new_folder_modal,
                min_visibility,
                on_close: move |_| all_folders.restart(),
                parent_folder_id,
            }

            if let Some(Ok(folders)) = &*all_folders.read() {
                if !folders.is_empty() {
                    div { class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3 mt-6",
                        for folder in folders {
                            Link {
                                class: "btn flex-col gap-2 normal-case p-2 h-auto",
                                to: Routes::folder(folder.id),
                                FolderOutline { class: "size-[90%]" }
                                {folder.name.clone()}
                            }
                        }
                    }
                } else {
                    div { class: "text-center mt-6", "This folder is empty" }
                }
            }
        }
    }
}

#[component]
pub fn LoggedIn(children: Element) -> Element {
    let is_logged_in = use_server_future(is_logged_in)?;

    rsx! {
        if let Some(Ok(true)) = is_logged_in() {
            {children}
        }
    }
}

#[component]
pub fn Modal(
    children: Element,
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

            div { class: "modal-box", {children} }

            if is_closable {
                div { class: "modal-backdrop", onclick: on_close }
            }
        }
    }
}

#[component]
pub fn NewFolderModal(
    mut is_open: Signal<bool>,
    #[props(default = FileVisibility::Private)] min_visibility: FileVisibility,
    on_close: Callback,
    parent_folder_id: Option<ReadOnlySignal<Uuid>>,
) -> Element {
    use_form_provider(attempt_to_create_folder);

    rsx! {
        FormSuccessModal { on_close }

        Modal { is_open,
            h2 { class: "h2", "New folder" }

            Form {
                on_success: move |_| {
                    *is_open.write() = false;
                },
                if let Some(parent_folder_id) = parent_folder_id {
                    input {
                        name: "parent_folder_id",
                        value: parent_folder_id().to_string(),
                        r#type: "hidden",
                    }
                }

                TextField { id: "name", label: "Name", name: "name" }

                SelectField {
                    id: "visibility",
                    label: "Visibility",
                    name: "visibility",
                    for (label , value) in FILE_VISIBILITY_OPTIONS
                        .iter()
                        .skip(
                            FILE_VISIBILITY_OPTIONS
                                .iter()
                                .position(|(_, value)| *value == min_visibility)
                                .unwrap_or_default(),
                        )
                    {
                        option { value: value.to_string(), {label} }
                    }
                }
            }
        }
    }
}

#[component]
pub fn PageTitle(children: Element) -> Element {
    let app_title = use_server_cached(|| app_title().unwrap_or("Lime3 (dev)".to_owned()));
    let vnode = children?;
    let page_title = match vnode.template {
        Template {
            roots: &[TemplateNode::Text { text }],
            node_paths: &[],
            attr_paths: &[],
            ..
        } => text.to_string(),
        Template {
            roots: &[TemplateNode::Dynamic { id }],
            node_paths: &[&[0]],
            attr_paths: &[],
            ..
        } => {
            let node = &vnode.dynamic_nodes[id];
            match node {
                DynamicNode::Text(text) => text.value.clone(),
                _ => {
                    return rsx!();
                }
            }
        }
        _ => {
            return rsx!();
        }
    };

    rsx! {
        document::Title { "{page_title} | {app_title}" }
    }
}

#[component]
pub fn RequireLogin(children: Element) -> Element {
    let is_logged_in = use_server_future(is_logged_in)?;
    let navigator = use_navigator();

    use_effect(move || {
        if let Some(Ok(false)) = is_logged_in() {
            navigator.push(Routes::login());
        }
    });

    rsx! {
        if let Some(Ok(true)) = is_logged_in() {
            {children}
        }
    }
}

#[component]
pub fn RequireNoLogin(children: Element) -> Element {
    let is_logged_in = use_server_future(is_logged_in)?;
    let navigator = use_navigator();

    use_effect(move || {
        if let Some(Ok(true)) = is_logged_in() {
            navigator.push(Routes::home());
        }
    });

    rsx! {
        if let Some(Ok(false)) = is_logged_in() {
            {children}
        }
    }
}
