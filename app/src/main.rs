use dioxus::prelude::*;

#[cfg(feature = "server")]
use axum::extract::{Path, Query};
#[cfg(feature = "server")]
use axum::response::IntoResponse;
#[cfg(feature = "server")]
use uuid::Uuid;

mod components;
mod constants;
mod forms;
mod hooks;
mod icons;
mod layouts;
mod pages;
mod presenters;
mod routes;
mod server_fns;
mod utils;

use hooks::use_resource_with_loader;
use routes::Routes;
use server_fns::get_current_user;
use utils::loader_is_active;

const FAVICON_ICO: Asset = asset!("assets/favicon.ico");
const ICON_SVG: Asset = asset!("assets/icon.svg");
const LOGO_SVG: Asset = asset!("assets/logo.svg");
const STYLE_CSS: Asset = asset!("assets/style.css");

#[cfg(feature = "server")]
#[derive(serde::Deserialize)]
struct FileQuery {
    width: Option<u16>,
    height: Option<u16>,
    fill: Option<bool>,
}

#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    use axum::routing::get;
    use tokio::net::TcpListener;

    dioxus::logger::initialize_default();

    let app = axum::Router::new()
        .route("/storage/files/{id}", get(get_storage_file))
        .serve_dioxus_application(ServeConfig::new().unwrap(), App);

    let addr = dioxus::cli_config::fullstack_address_or_localhost();
    let listener = TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app.into_make_service()).await.unwrap();
}

#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(App);
}

#[cfg(feature = "server")]
async fn get_storage_file(Path(id): Path<Uuid>, Query(query): Query<FileQuery>) -> impl IntoResponse {
    use axum::body::Body;
    use axum::http::StatusCode;
    use axum::http::header::{CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE};

    let file = drive_core::server::commands::get_file_by_id(id, None)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "FILE NOT FOUND"))?;

    let Some(content) = file.read_variant(query.width, query.height, query.fill) else {
        return Err((StatusCode::FORBIDDEN, "FORBIDDEN"));
    };

    let content_length = content.len();
    let body = Body::from(content);

    let headers = [
        (CONTENT_TYPE, file.media_type.to_string()),
        (CONTENT_LENGTH, content_length.to_string()),
        (
            CONTENT_DISPOSITION,
            format!(
                "inline; filename=\"{}\"",
                file.variant_filename(query.width, query.height, query.fill)
            ),
        ),
    ];

    Ok((headers, body))
}

#[component]
fn App() -> Element {
    let current_user = use_resource_with_loader("current-user".to_owned(), async || {
        get_current_user().await.ok().flatten()
    });
    let mut app_is_loading = use_signal(|| true);

    use_context_provider(|| current_user);

    use_effect(move || {
        if current_user.read().is_some() {
            app_is_loading.set(false);
        }
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON_ICO }
        document::Link { rel: "stylesheet", href: STYLE_CSS }

        Router::<Routes> {}

        div {
            class: "loading loading-spinner loading-xl fixed bottom-3 right-3",
            class: if !loader_is_active() { "hidden" },
        }

        div {
            class: "loading-overlay",
            class: if !app_is_loading() { "loading-overlay-hidden" },
            figure {
                div { class: "loading-overlay-pulse" }
                img { src: ICON_SVG }
            }
        }
    }
}
