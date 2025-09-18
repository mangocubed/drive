use dioxus::prelude::*;
use uuid::Uuid;

use crate::components::{FileMenu, PageTitle};
use crate::hooks::use_resource_with_loader;
use crate::routes::Routes;
use crate::server_fns::get_file;

#[component]
pub fn FilePage(id: ReadOnlySignal<Uuid>) -> Element {
    let navigator = use_navigator();
    let file = use_resource_with_loader(
        "file".to_owned(),
        move || async move { get_file(id()).await.ok().flatten() },
    );
    let page_title = use_memo(move || {
        if let Some(Some(file)) = &*file.read() {
            let mut title = "Home > ".to_owned();

            if !file.parent_folders.is_empty() {
                title += &file
                    .parent_folders
                    .clone()
                    .iter()
                    .map(|(_, name)| name.clone())
                    .collect::<Vec<_>>()
                    .join(" > ");
                title += " > ";
            }

            Some(title + &file.name)
        } else {
            None
        }
    });

    rsx! {
        if let Some(Some(file)) = &*file.read() {
            PageTitle { {page_title()} }

            h1 { class: "h2 breadcrumbs",
                ul {
                    li {
                        Link { to: Routes::home(), "Home" }
                    }
                    for (id , name) in file.parent_folders.clone() {
                        li {
                            Link { to: Routes::folder(id), {name.clone()} }
                        }
                    }
                    li { {file.name.clone()} }
                }
            }

            div { class: "text-right",
                FileMenu {
                    file: file.clone(),
                    on_trashed: move |_| {
                        navigator.push(Routes::home());
                    },
                }
            }

            div { class: "my-4",
                img {
                    class: "m-auto max-h-[calc(100vh-2rem)]",
                    src: file.preview_url.clone(),
                    alt: file.name.clone(),
                }
            }
        }
    }
}
