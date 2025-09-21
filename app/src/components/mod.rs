use dioxus::core::{DynamicNode, Template, TemplateNode};
use dioxus::prelude::*;

use crate::components::modals::RenameFileModal;
use crate::icons::{ArrowDownTrayOutline, EllipsisVerticalOutline, PencilOutline, TrashOutline};
use crate::presenters::{FilePresenter, FolderPresenter};
use crate::server_fns::{attempt_to_move_file_to_trash, attempt_to_move_folder_to_trash, is_logged_in};
use crate::utils::run_with_loader;
use crate::{ICON_SVG, LOGO_SVG, use_resource_with_loader};

mod file_manager;
mod modals;

pub use file_manager::FileManager;
pub use modals::{AboutModal, ConfirmationModal, Modal, SubscriptionModal};

#[component]
pub fn FileMenu(#[props(into)] file: FilePresenter, #[props(into)] on_update: Callback) -> Element {
    let mut show_rename_modal = use_signal(|| false);
    let mut show_trash_confirmation = use_signal(|| false);

    rsx! {
        div { class: "dropdown dropdown-end",
            button { class: "btn btn-outline btn-square", tabindex: 0, EllipsisVerticalOutline {} }

            ul {
                class: "menu menu-sm dropdown-content bg-base-200 rounded-box shadow mt-3 p-2 w-max z-1",
                tabindex: 0,
                li {
                    a { download: file.name.clone(), href: file.url.clone(),
                        ArrowDownTrayOutline {}
                        "Download"
                    }
                }

                div { class: "divider m-1" }

                li {
                    a {
                        onclick: move |_| {
                            *show_rename_modal.write() = true;
                        },
                        PencilOutline {}
                        "Rename"
                    }
                }

                div { class: "divider m-1" }

                li {
                    a {
                        onclick: move |_| {
                            *show_trash_confirmation.write() = true;
                        },
                        TrashOutline {}
                        "Move to trash"
                    }
                }
            }
        }

        RenameFileModal {
            is_open: show_rename_modal,
            file: file.clone(),
            on_close: move |_| {
                on_update.call(());
            },
        }

        ConfirmationModal {
            is_open: show_trash_confirmation,
            on_accept: {
                let file_id = file.id;
                move |_| {
                    async move {
                        let result = run_with_loader(

                                "move-file-to-trash".to_owned(),
                                move || attempt_to_move_file_to_trash(file_id),
                            )
                            .await;
                        if result.is_ok() {
                            on_update.call(());
                        }
                    }
                }
            },
            "Are you sure you want to move this file to trash?"
        }
    }
}

#[component]
pub fn FolderMenu(#[props(into)] folder: FolderPresenter, #[props(into)] on_update: Callback) -> Element {
    let mut show_trash_confirmation = use_signal(|| false);

    rsx! {
        div { class: "dropdown dropdown-end",
            button { class: "btn btn-outline btn-square", tabindex: 0, EllipsisVerticalOutline {} }

            ul {
                class: "menu menu-sm dropdown-content bg-base-200 rounded-box shadow mt-3 p-2 w-max z-1",
                tabindex: 0,

                li {
                    a {
                        onclick: move |_| {
                            *show_trash_confirmation.write() = true;
                        },
                        TrashOutline {}
                        "Move to trash"
                    }
                }
            }
        }


        ConfirmationModal {
            is_open: show_trash_confirmation,
            on_accept: {
                let folder_id = folder.id;
                move |()| {
                    async move {
                        let result = run_with_loader(

                                "move-folder-to-trash".to_owned(),
                                move || attempt_to_move_folder_to_trash(folder_id),
                            )
                            .await;
                        if result.is_ok() {
                            on_update.call(());
                        }
                    }
                }
            },
            "Are you sure you want to move this folder to trash?"
        }
    }
}

#[component]
pub fn LoggedIn(children: Element) -> Element {
    let is_logged_in = use_resource_with_loader("logged-in".to_owned(), is_logged_in);

    rsx! {
        if let Some(Ok(true)) = is_logged_in() {
            {children}
        }
    }
}

#[component]
pub fn Brand() -> Element {
    rsx! {
        div { class: "flex gap-2 items-center",
            img { class: "h-[36px] sm:hidden", src: ICON_SVG }

            img { class: "h-[36px] max-sm:hidden", src: LOGO_SVG }

            div { class: "text-3xl font-bold opacity-80", "Drive" }

            if cfg!(debug_assertions) {
                div { class: "text-sm opacity-70 self-start", "(dev)" }
            }
        }
    }
}

#[component]
pub fn PageTitle(children: Element) -> Element {
    let app_title = use_server_cached(|| {
        let app_title = dioxus::cli_config::app_title().unwrap_or("Mango³ Drive".to_owned());

        if cfg!(debug_assertions) {
            app_title + " (dev)"
        } else {
            app_title
        }
    });

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
