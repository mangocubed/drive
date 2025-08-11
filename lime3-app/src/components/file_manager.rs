use dioxus::prelude::*;
use uuid::Uuid;

use lime3_core::enums::FileVisibility;
use lime3_core::inputs::FileInput;

use crate::forms::{Form, FormSuccessModal, SelectField, TextField, use_form_provider};
use crate::icons::{
    ArrowUpTrayOutline, CheckCircleOutline, ExclamationTriangleOutline, FolderOutline, FolderPlusOutline,
};
use crate::routes::Routes;
use crate::server_functions::{attempt_to_create_folder, attempt_to_upload_file, get_all_folder_items};

use super::{LoggedIn, Modal};

const FILE_VISIBILITY_OPTIONS: [(&str, FileVisibility); 4] = [
    ("Private", FileVisibility::Private),
    ("Only followers", FileVisibility::Followers),
    ("Only users", FileVisibility::Users),
    ("Public", FileVisibility::Public),
];

#[component]
pub fn FileManager(
    #[props(default = FileVisibility::Private)] min_visibility: FileVisibility,
    #[props(optional)] parent_folder_id: ReadOnlySignal<Option<Uuid>>,
) -> Element {
    let mut show_new_folder_modal = use_signal(|| false);
    let mut pending_files = use_signal(Vec::new);
    let mut all_folder_items = use_server_future(move || get_all_folder_items(parent_folder_id()))?;

    rsx! {
        LoggedIn {
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
                                                parent_folder_id: parent_folder_id(),
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

            NewFolderModal {
                is_open: show_new_folder_modal,
                min_visibility,
                on_close: move |_| all_folder_items.restart(),
                parent_folder_id,
            }

            UploadFilesModal {
                files: pending_files,
                on_close: move |_| all_folder_items.restart(),
            }

            if let Some(Ok(folder_items)) = &*all_folder_items.read() {
                if !folder_items.is_empty() {
                    div { class: "grid grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3 mt-6",
                        for folder_item in folder_items {
                            if folder_item.is_file {
                                Link {
                                    class: "btn flex-col gap-2 p-2 h-auto",
                                    to: Routes::file(folder_item.id),
                                    img {
                                        class: "rounded-lg m-auto",
                                        src: folder_item.preview_url.clone(),
                                    }
                                    div { class: "normal-case truncate w-full",
                                        {folder_item.name.clone()}
                                    }
                                }
                            } else {
                                Link {
                                    class: "btn flex-col gap-2 normal-case p-2 h-auto",
                                    to: Routes::folder(folder_item.id),
                                    FolderOutline { class: "size-[90%]" }
                                    div { class: "normal-case truncate w-full",
                                        {folder_item.name.clone()}
                                    }
                                }
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
fn NewFolderModal(
    mut is_open: Signal<bool>,
    #[props(default = FileVisibility::Private)] min_visibility: FileVisibility,
    on_close: Callback,
    parent_folder_id: ReadOnlySignal<Option<Uuid>>,
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
                        option { value: value.to_string(), {label} }
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
