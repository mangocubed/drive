use dioxus::prelude::*;

use crate::components::{ConfirmationModal, PageTitle};
use crate::hooks::{use_current_user, use_resource_with_loader};
use crate::icons::{EllipsisVerticalOutline, FolderOutline};
use crate::presenters::FolderItemPresenter;
use crate::server_fns::{
    attempt_to_empty_trash, attempt_to_restore_file, attempt_to_restore_folder, get_all_trash_items,
};
use crate::utils::{loader_is_active, run_with_loader};

#[component]
pub fn TrashItemMenu(trash_item: FolderItemPresenter, #[props(into)] on_restore: Callback) -> Element {
    rsx! {
        div { class: "absolute top-0.5 right-0.5",
            div { class: "dropdown dropdown-end absolute top-1 right-1",
                button {
                    class: "btn btn-outline btn-square",
                    tabindex: 0,
                    onclick: move |event| event.prevent_default(),
                    EllipsisVerticalOutline {}
                }

                ul {
                    class: "menu menu-sm dropdown-content bg-base-200 rounded-box shadow mt-3 p-2 w-max z-1",
                    tabindex: 0,
                    li {
                        a {
                            onclick: {
                                let id = trash_item.id;

                                move |_| async move {
                                    if trash_item.is_file {
                                        let _ = run_with_loader(
                                                "restore-file".to_owned(),
                                                move || attempt_to_restore_file(id),
                                            )
                                            .await;
                                    } else {
                                        let _ = run_with_loader(
                                                "restore-folder".to_owned(),
                                                move || attempt_to_restore_folder(id),
                                            )
                                            .await;
                                    }
                                    on_restore.call(());
                                }
                            },
                            "Restore"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn TrashPage() -> Element {
    let mut all_trash_items = use_resource_with_loader("trash-items".to_owned(), get_all_trash_items);
    let mut current_user = use_current_user();
    let trash_items_count = use_memo(move || {
        all_trash_items
            .read()
            .as_ref()
            .and_then(|result| result.as_ref().ok())
            .map_or(0, |trash_items| trash_items.len())
    });
    let mut show_empty_confirmation = use_signal(|| false);

    rsx! {
        PageTitle { "Trash" }

        h1 { class: "h1", "Trash" }

        div { class: "text-right",
            button {
                class: "btn btn-outline",
                disabled: trash_items_count() == 0 || loader_is_active(),
                onclick: move |event| {
                    event.prevent_default();

                    *show_empty_confirmation.write() = true;
                },
                "Empty trash"
            }
        }

        if let Some(Ok(trash_items)) = &*all_trash_items.read() {
            if !trash_items.is_empty() {
                div { class: "grid grid-cols-3 sm:grid-cols-4 lg:grid-cols-5 gap-3 mt-6",
                    for trash_item in trash_items {
                        div { class: "relative",
                            if trash_item.is_file {
                                div { class: "btn flex-col gap-2 p-2 h-full w-full",
                                    img {
                                        class: "rounded-lg m-auto",
                                        src: trash_item.preview_url.clone(),
                                    }

                                    div { class: "normal-case truncate w-full",
                                        {trash_item.name.clone()}
                                    }
                                }
                            } else {
                                div { class: "btn flex-col gap-2 normal-case p-2 h-full w-full",
                                    FolderOutline { class: "size-[90%]" }
                                    div { class: "normal-case truncate w-full",
                                        {trash_item.name.clone()}
                                    }
                                }
                            }

                            TrashItemMenu {
                                trash_item: trash_item.clone(),
                                on_restore: move |_| {
                                    current_user.restart();
                                    all_trash_items.restart();
                                },
                            }
                        }
                    }
                }

                ConfirmationModal {
                    is_open: show_empty_confirmation,
                    on_accept: move |_| async move {
                        let _ = run_with_loader("empty-trash".to_owned(), attempt_to_empty_trash).await;
                        current_user.restart();
                        all_trash_items.restart();
                    },
                    "Are you sure you want to empty the trash?"
                }
            } else {
                div { class: "text-center mt-6", "Trash is empty" }
            }
        }
    }
}
