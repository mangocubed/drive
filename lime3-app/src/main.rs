use dioxus::prelude::*;

#[cfg(feature = "server")]
use axum::extract::{Path, Query};
#[cfg(feature = "server")]
use axum::response::IntoResponse;
#[cfg(feature = "server")]
use uuid::Uuid;

mod components;
mod forms;
mod icons;
mod layouts;
mod pages;
mod presenters;
mod routes;
mod server_fns;
mod utils;

use presenters::UserPresenter;
use routes::Routes;
use server_fns::get_current_user;

const FAVICON_ICO: Asset = asset!("assets/favicon.ico");
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

    let file = lime3_core::server::commands::get_file_by_id(id, None)
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

fn use_current_user() -> Resource<Option<UserPresenter>> {
    use_context()
}

#[component]
fn App() -> Element {
    let current_user = use_resource(async || get_current_user().await.ok().flatten());

    use_context_provider(|| current_user);

    rsx! {
        document::Link { rel: "icon", href: FAVICON_ICO }
        document::Link { rel: "stylesheet", href: STYLE_CSS }

        Router::<Routes> {}
    }
}
