use dioxus::prelude::*;
use uuid::Uuid;

use crate::components::PageTitle;
use crate::icons::ArrowDownTrayOutline;
use crate::routes::Routes;
use crate::server_functions::get_file;

#[component]
pub fn FilePage(id: ReadOnlySignal<Uuid>) -> Element {
    let file = use_server_future(move || async move { get_file(id()).await.ok().flatten() })?;
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

            h1 { class: "h1 breadcrumbs",
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

            div { class: "my-4",
                img {
                    class: "m-auto max-h-[calc(100vh-2rem)]",
                    src: file.preview_url.clone(),
                    alt: file.name.clone(),
                }
            }

            div { class: "flex justify-center",
                a {
                    class: "btn btn-outline m-auto",
                    download: file.name.clone(),
                    href: file.url.clone(),
                    ArrowDownTrayOutline {}
                    "Download"
                }
            }
        }
    }
}
