use dioxus::prelude::*;
use serde_json::Value;
use uuid::Uuid;

use sdk::components::{Form, FormSuccessModal, Modal, SelectField, TextField};
use sdk::hooks::{use_form_provider, use_resource_with_loader};
use sdk::run_with_loader;

use drive_core::enums::FileVisibility;
use drive_core::inputs::FileInput;

use crate::components::FolderItemMenu;
use crate::hooks::use_current_user;
use crate::icons::{
    ArrowUpTrayOutline, CheckCircleOutline, ExclamationTriangleOutline, FolderOutline, FolderPlusOutline, MoveOutline,
};
use crate::presenters::FolderPresenter;
use crate::routes::Routes;
use crate::server_fns::{
    attempt_to_create_folder, attempt_to_move_file, attempt_to_move_folder, attempt_to_upload_file,
    get_all_folder_items,
};
use crate::signals::MOVE_FOLDER_ITEM;
use crate::utils::can_be_moved;

const FILE_VISIBILITY_OPTIONS: [(&str, FileVisibility); 4] = [
    ("Private", FileVisibility::Private),
    ("Only followers", FileVisibility::Followers),
    ("Only users", FileVisibility::Users),
    ("Public", FileVisibility::Public),
];

#[component]
pub fn FileManager(
    #[props(default = FileVisibility::Private)] min_visibility: FileVisibility,
    #[props(optional)] folder: ReadSignal<Option<FolderPresenter>>,
) -> Element {
    let navigator = use_navigator();
    let folder_id = use_memo(move || folder().map(|folder| folder.id));
    let mut show_new_folder_modal = use_signal(|| false);
    let mut pending_files = use_signal(Vec::new);
    let mut all_folder_items = use_resource_with_loader("folder-items", move || get_all_folder_items(folder_id()));
    let mut current_user = use_current_user();

    rsx! {
        div { class: "flex gap-2 justify-between",
            div { class: "join",
                button {
                    class: "btn btn-outline join-item",
                    onclick: move |_| show_new_folder_modal.set(true),
                    FolderPlusOutline {}
                    "New folder"
                }

                label { class: "btn btn-outline join-item",
                    input {
                        accept: "image/bmp,image/gif,image/jpeg,image/png,image/webp",
                        class: "hidden",
                        r#type: "file",
                        multiple: true,
                        onchange: move |event| {
                            event.prevent_default();

                            async move {
                                if let Some(file_engine) = event.files() {
                                    for file_name in file_engine.files() {
                                        pending_files
                                            .write()
                                            .push(FileInput {
                                                parent_folder_id: folder_id(),
                                                name: file_name.clone(),
                                                content: file_engine.read_file(&file_name).await.unwrap(),
                                            });
                                    }
                                }
                            }
                        },
                    }

                    ArrowUpTrayOutline {}

                    "Upload files"
                }
            }

            if let Some(folder) = folder() {
                FolderItemMenu {
                    folder_item: folder,
                    on_update: move |_| {
                        navigator.push(Routes::home());
                    },
                }
            }
        }

        NewFolderModal {
            is_open: show_new_folder_modal,
            min_visibility,
            on_close: move |_| all_folder_items.restart(),
            parent_folder_id: folder_id,
        }

        UploadFilesModal {
            files: pending_files,
            on_close: move |_| {
                current_user.restart();
                all_folder_items.restart();
            },
        }

        if let Some(Ok(folder_items)) = &*all_folder_items.read() {
            if !folder_items.is_empty() {
                div { class: "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3 mt-6",
                    for folder_item in folder_items {
                        div { class: "relative aspect-square",
                            if folder_item.is_file {
                                Link {
                                    class: "btn flex-col gap-2 p-2 h-full w-full",
                                    to: Routes::file(folder_item.id),
                                    img {
                                        class: "rounded-lg m-auto min-h-0",
                                        src: folder_item.variant_url(200, 200, false).unwrap().to_string(),
                                    }
                                    div { class: "normal-case truncate w-full shrink-0",
                                        {folder_item.name.clone()}
                                    }
                                }

                                div { class: "absolute top-0.5 right-0.5",
                                    FolderItemMenu {
                                        folder_item: folder_item.clone(),
                                        on_update: move |_| all_folder_items.restart(),
                                    }
                                }
                            } else {
                                Link {
                                    class: "btn flex-col gap-2 normal-case p-2 h-full w-full",
                                    to: Routes::folder(folder_item.id),
                                    FolderOutline { class: "size-[90%] text-gray-400 hover:text-gray-200" }

                                    div { class: "normal-case truncate w-full shrink-0",
                                        {folder_item.name.clone()}
                                    }
                                }

                                div { class: "absolute top-0.5 right-0.5",
                                    FolderItemMenu {
                                        folder_item: folder_item.clone(),
                                        on_update: move |_| all_folder_items.restart(),
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "text-center mt-6", "This folder is empty" }
            }
        }

        if let Some(move_folder_item) = &*MOVE_FOLDER_ITEM.read() {
            div { class: "fixed bottom-4 flex",
                div { role: "alert", class: "alert",
                    MoveOutline {}

                    "Moving \""
                    {move_folder_item.name.clone()}
                    "\""

                    button {
                        class: "btn btn-sm btn-outline",
                        onclick: move |event| {
                            event.prevent_default();
                            *MOVE_FOLDER_ITEM.write() = None;
                        },
                        "Cancel"
                    }

                    button {
                        class: "btn btn-sm btn-primary",
                        disabled: !can_be_moved(move_folder_item, folder().as_ref()),
                        onclick: {
                            let move_folder_item_id = move_folder_item.id;
                            let move_folder_item_is_file = move_folder_item.is_file;
                            move |event| {
                                event.prevent_default();
                                async move {
                                    let result = if move_folder_item_is_file {
                                        run_with_loader(
                                                "move-file",
                                                move || attempt_to_move_file(
                                                    move_folder_item_id,
                                                    folder_id(),
                                                ),
                                            )
                                            .await
                                    } else {
                                        run_with_loader(
                                                "move-folder",
                                                move || attempt_to_move_folder(
                                                    move_folder_item_id,
                                                    folder_id(),
                                                ),
                                            )
                                            .await
                                    };
                                    if result.is_ok() {
                                        *MOVE_FOLDER_ITEM.write() = None;
                                        all_folder_items.restart();
                                    }
                                }
                            }
                        },
                        "Paste here"
                    }
                }
            }
        }
    }
}

#[component]
fn NewFolderModal(
    mut is_open: Signal<bool>,
    #[props(default = FileVisibility::Private)] min_visibility: FileVisibility,
    on_close: Callback<Value>,
    parent_folder_id: ReadSignal<Option<Uuid>>,
) -> Element {
    use_form_provider("create-folder", attempt_to_create_folder);

    rsx! {
        FormSuccessModal { on_close }

        Modal { is_open,
            h2 { class: "h2", "New folder" }

            Form {
                on_success: move |_| {
                    *is_open.write() = false;
                },
                if let Some(parent_folder_id) = parent_folder_id() {
                    input {
                        name: "parent_folder_id",
                        value: parent_folder_id.to_string(),
                        r#type: "hidden",
                    }
                }

                TextField { id: "name", label: "Name", name: "name" }

                SelectField {
                    id: "visibility",
                    label: "Visibility",
                    name: "visibility",
                    for (label , value) in FILE_VISIBILITY_OPTIONS.iter().skip_while(|(_, value)| *value != min_visibility) {
                        option { value: value.to_string(), {*label} }
                    }
                }
            }
        }
    }
}

#[component]
pub fn UploadFilesModal(files: Signal<Vec<FileInput>>, on_close: Callback) -> Element {
    let mut is_open = use_signal(|| false);
    let mut uploads_result = use_signal(Vec::new);

    use_effect(move || {
        if files().is_empty() {
            return;
        }

        *is_open.write() = true;

        spawn(async move {
            for file in files() {
                uploads_result.write().push(attempt_to_upload_file(file).await);
            }
        });
    });

    rsx! {
        Modal { is_open, is_closable: false,
            h2 { class: "h2", "Uploading Files" }

            div { class: "flex flex-col gap-2",
                for (index , file) in files().iter().enumerate() {
                    div { class: "card card-sm card-border",
                        div { class: "card-body flex-row justify-between",
                            div { class: "font-bold", {file.name.clone()} }

                            match uploads_result().get(index) {
                                Some(Ok(true)) => rsx! {
                                    div { CheckCircleOutline {} }
                                },
                                Some(Ok(false)) | Some(Err(_)) => rsx! {
                                    div { ExclamationTriangleOutline {} }
                                },
                                _ => rsx! {
                                    div { class: "loading loading-spinner" }
                                },
                            }
                        }
                    }
                }
            }

            div { class: "modal-action",
                button {
                    class: "btn",
                    disabled: uploads_result().len() < files().len(),
                    onclick: move |event| {
                        event.prevent_default();
                        files.write().clear();
                        uploads_result.write().clear();
                        *is_open.write() = false;
                        on_close.call(());
                    },
                    "Close"
                }
            }
        }
    }
}
