use dioxus::core::{DynamicNode, Template, TemplateNode};
use dioxus::prelude::*;

use crate::components::modals::RenameModal;
use crate::icons::*;
use crate::presenters::{FilePresenter, FolderItemPresenter};
use crate::server_fns::*;
use crate::signals::MOVE_FOLDER_ITEM;
use crate::utils::{can_be_moved, run_with_loader};
use crate::{ICON_SVG, LOGO_SVG, use_resource_with_loader};

mod file_manager;
mod modals;

pub use file_manager::FileManager;
pub use modals::{AboutModal, ConfirmationModal, Modal, SubscriptionModal};

#[cfg(feature = "web")]
#[component]
fn DownloadLink(#[props(into)] file: FilePresenter) -> Element {
    use dioxus::web::WebEventExt;
    use web_sys::HtmlAnchorElement;
    use web_sys::wasm_bindgen::JsCast;

    let mut download_url = use_signal(|| None);
    let mut download_el: Signal<Option<HtmlAnchorElement>> = use_signal(|| None);

    use_effect(move || {
        if let Some(el) = download_el()
            && download_url().is_some()
        {
            el.click();
        }
    });

    rsx! {
        a {
            onclick: move |_| {
                async move {
                    let result = run_with_loader(
                            "get-file-url".to_owned(),
                            move || get_file_url(file.id),
                        )
                        .await;

                    if let Ok(mut file_url) = result {
                        file_url.set_query(Some("download=true"));

                        *download_url.write() = Some(file_url);
                    }
                }
            },
            ArrowDownTrayOutline {}
            "Download"
        }

        a {
            onmounted: move |event| {
                *download_el.write() = event
                    .data()
                    .as_web_event()
                    .dyn_into::<HtmlAnchorElement>()
                    .ok();
            },
            class: "hidden",
            download: file.name.clone(),
            href: download_url().map(|url| url.to_string()),
        }
    }
}

#[cfg(any(feature = "desktop", feature = "mobile"))]
#[component]
fn DownloadLink(#[props(into)] file: FilePresenter) -> Element {
    rsx! {
        a {
            onclick: move |_| {
                async move {
                    let result = run_with_loader(
                            "get-file-url".to_owned(),
                            move || get_file_url(file.id),
                        )
                        .await;

                    if let Ok(mut file_url) = result {
                        file_url.set_query(Some("download=true"));

                        #[cfg(feature = "desktop")]
                        let _ = dioxus::desktop::use_window()
                            .webview
                            .load_url(file_url.as_ref());

                        #[cfg(feature = "mobile")]
                        let _ = dioxus::mobile::use_window().webview.load_url(file_url.as_ref());
                    }
                }
            },
            ArrowDownTrayOutline {}
            "Download"
        }
    }
}

#[cfg(feature = "server")]
#[component]
fn DownloadLink(#[props(into)] file: FilePresenter) -> Element {
    VNode::empty()
}

#[component]
pub fn FolderItemMenu(#[props(into)] folder_item: FolderItemPresenter, #[props(into)] on_update: Callback) -> Element {
    let mut show_rename_modal = use_signal(|| false);
    let mut show_trash_confirmation = use_signal(|| false);

    rsx! {
        div { class: "dropdown dropdown-end",
            button { class: "btn btn-outline btn-square", tabindex: 0, EllipsisVerticalOutline {} }

            ul {
                class: "menu menu-sm dropdown-content bg-base-200 rounded-box shadow mt-3 p-2 w-max z-1",
                tabindex: 0,
                if folder_item.is_file {
                    li {
                        DownloadLink { file: folder_item.clone() }
                    }

                    div { class: "divider m-1" }
                }

                li {
                    a {
                        onclick: move |_| {
                            *show_rename_modal.write() = true;
                        },
                        PencilOutline {}
                        "Rename"
                    }
                }

                li {
                    a {
                        onclick: move |_| {
                            *MOVE_FOLDER_ITEM.write() = Some(folder_item.clone());
                        },
                        MoveOutline {}
                        "Move"
                    }
                }

                if !folder_item.is_file && let Some(move_folder_item) = &*MOVE_FOLDER_ITEM.read()
                    && can_be_moved(move_folder_item, Some(&(&folder_item).into()))
                {
                    li {
                        a {
                            onclick: {
                                let move_folder_item_id = move_folder_item.id;
                                let move_folder_item_is_file = move_folder_item.is_file;
                                let target_folder_id = folder_item.id;
                                move |_| {
                                    async move {
                                        let result = if move_folder_item_is_file {
                                            run_with_loader(
                                                    "move-file".to_owned(),
                                                    move || attempt_to_move_file(
                                                        move_folder_item_id,
                                                        Some(target_folder_id),
                                                    ),
                                                )
                                                .await
                                        } else {
                                            run_with_loader(
                                                    "move-folder".to_owned(),
                                                    move || attempt_to_move_folder(
                                                        move_folder_item_id,
                                                        Some(target_folder_id),
                                                    ),
                                                )
                                                .await
                                        };
                                        if result.is_ok() {
                                            *MOVE_FOLDER_ITEM.write() = None;
                                            on_update.call(());
                                        }
                                    }
                                }
                            },
                            ClipboardDocumentListOutline {}
                            "Paste here"
                        }
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

        RenameModal {
            is_open: show_rename_modal,
            folder_item: folder_item.clone(),
            on_success: move |_| {
                on_update.call(());
            },
        }

        ConfirmationModal {
            is_open: show_trash_confirmation,
            on_accept: {
                let folder_item_id = folder_item.id;
                move |_| {
                    async move {
                        let result = if folder_item.is_file {
                            run_with_loader(
                                    "move-file-to-trash".to_owned(),
                                    move || attempt_to_move_file_to_trash(folder_item_id),
                                )
                                .await
                        } else {
                            run_with_loader(
                                    "move-folder-to-trash".to_owned(),
                                    move || attempt_to_move_folder_to_trash(folder_item_id),
                                )
                                .await
                        };
                        if result.is_ok() {
                            on_update.call(());
                        }
                    }
                }
            },
            "Are you sure you want to move this "
            if folder_item.is_file {
                "file"
            } else {
                "folder"
            }
            " to trash?"
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
