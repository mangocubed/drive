use dioxus::prelude::*;
use uuid::Uuid;

use sdk::components::PageTitle;

use crate::components::FileManager;
use crate::routes::Routes;
use crate::server_fns::get_folder;
use crate::use_resource_with_loader;

#[component]
pub fn FolderPage(id: ReadSignal<Uuid>) -> Element {
    let folder = use_resource_with_loader("folder", move || async move { get_folder(id()).await.ok().flatten() });
    let page_title = use_memo(move || {
        if let Some(Some(folder)) = &*folder.read() {
            let mut title = "Home > ".to_owned();

            if !folder.parent_folders.is_empty() {
                title += &folder
                    .parent_folders
                    .clone()
                    .iter()
                    .map(|parent_folder| parent_folder.name.clone())
                    .collect::<Vec<_>>()
                    .join(" > ");
                title += " > ";
            }

            Some(title + &folder.name)
        } else {
            None
        }
    });

    rsx! {
        if let Some(Some(folder)) = &*folder.read() {
            PageTitle { {page_title()} }

            h1 { class: "h3 breadcrumbs",
                ul {
                    li {
                        Link { to: Routes::home(), "Home" }
                    }
                    for parent_folder in folder.parent_folders.clone() {
                        li {
                            Link { to: Routes::folder(parent_folder.id), {parent_folder.name.clone()} }
                        }
                    }
                    li { {folder.name.clone()} }
                }
            }

            FileManager {
                min_visibility: folder.visibility,
                folder: Some(folder.clone()),
            }
        }
    }
}
