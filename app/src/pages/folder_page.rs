use dioxus::prelude::*;
use uuid::Uuid;

use crate::components::{FileManager, PageTitle};
use crate::routes::Routes;
use crate::server_fns::get_folder;
use crate::use_resource_with_loader;

#[component]
pub fn FolderPage(id: ReadOnlySignal<Uuid>) -> Element {
    let folder = use_resource_with_loader("folder".to_owned(), move || async move {
        get_folder(id()).await.ok().flatten()
    });
    let page_title = use_memo(move || {
        if let Some(Some(folder)) = &*folder.read() {
            let mut title = "Home > ".to_owned();

            if !folder.parent_folders.is_empty() {
                title += &folder
                    .parent_folders
                    .clone()
                    .iter()
                    .map(|(_, name)| name.clone())
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

            h1 { class: "h2 breadcrumbs",
                ul {
                    li {
                        Link { to: Routes::home(), "Home" }
                    }
                    for (id , name) in folder.parent_folders.clone() {
                        li {
                            Link { to: Routes::folder(id), {name.clone()} }
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
