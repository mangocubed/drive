use dioxus::cli_config::app_title;
use dioxus::core::{DynamicNode, Template, TemplateNode};
use dioxus::prelude::*;

use crate::routes::Routes;
use crate::server_functions::is_logged_in;

mod file_manager;

pub use file_manager::FileManager;

#[component]
pub fn ConfirmationModal(children: Element, is_open: Signal<bool>, on_accept: Callback) -> Element {
    rsx! {
        Modal { is_closable: false, is_open,
            div { {children} }

            div { class: "modal-action",
                button {
                    class: "btn",
                    onclick: move |event| {
                        event.prevent_default();
                        *is_open.write() = false;
                    },
                    "Cancel"
                }
                button {
                    class: "btn btn-primary",
                    onclick: move |event| {
                        event.prevent_default();
                        *is_open.write() = false;
                        on_accept.call(());
                    },
                    "Accept"
                }
            }
        }
    }
}

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
pub fn Modal(
    children: Element,
    is_open: Signal<bool>,
    #[props(default = true)] is_closable: bool,
    #[props(optional)] on_close: Callback<MouseEvent>,
) -> Element {
    let on_close = move |event: MouseEvent| {
        event.prevent_default();
        *is_open.write() = false;
        on_close.call(event);
    };

    rsx! {
        dialog { class: "modal", class: if is_open() { "modal-open" },
            if is_closable {
                button {
                    class: "btn btn-sm btn-circle btn-ghost absolute right-2 top-2",
                    onclick: on_close,
                    "✕"
                }
            }

            div { class: "modal-box", {children} }

            if is_closable {
                div { class: "modal-backdrop", onclick: on_close }
            }
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

#[component]
pub fn RequireLogin(children: Element) -> Element {
    let is_logged_in = use_server_future(is_logged_in)?;
    let navigator = use_navigator();

    use_effect(move || {
        if let Some(Ok(false)) = is_logged_in() {
            navigator.push(Routes::login());
        }
    });

    rsx! {
        if let Some(Ok(true)) = is_logged_in() {
            {children}
        }
    }
}

#[component]
pub fn RequireNoLogin(children: Element) -> Element {
    let is_logged_in = use_server_future(is_logged_in)?;
    let navigator = use_navigator();

    use_effect(move || {
        if let Some(Ok(true)) = is_logged_in() {
            navigator.push(Routes::home());
        }
    });

    rsx! {
        if let Some(Ok(false)) = is_logged_in() {
            {children}
        }
    }
}
