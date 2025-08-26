use dioxus::cli_config::app_title;
use dioxus::core::{DynamicNode, Template, TemplateNode};
use dioxus::prelude::*;

use crate::server_functions::is_logged_in;

mod file_manager;
mod modals;

pub use file_manager::FileManager;
pub use modals::{ConfirmationModal, EditSpaceModal, Modal};

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
