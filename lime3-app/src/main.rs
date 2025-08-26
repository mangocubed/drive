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
mod server_functions;

#[cfg(feature = "server")]
mod server;

use presenters::UserPresenter;
use routes::Routes;
use server_functions::get_current_user;

const FAVICON_ICO: Asset = asset!("assets/favicon.ico");
const STYLE_CSS: Asset = asset!("assets/style.css");

#[cfg(feature = "server")]
#[derive(serde::Deserialize)]
pub struct FileQuery {
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub fill: Option<bool>,
}

#[cfg(feature = "server")]
#[derive(serde::Deserialize)]
pub struct CheckoutQuery {
    pub checkout_id: Uuid,
}

#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    use axum::routing::get;
    use cookie::{Key, SameSite};
    use fred::prelude::{ClientLike, Config, Pool};
    use time::Duration;
    use tokio::net::TcpListener;
    use tower_sessions::{Expiry, SessionManagerLayer};
    use tower_sessions_redis_store::RedisStore;

    use lime3_core::server::config::SESSION_CONFIG;

    dioxus::logger::initialize_default();

    let redis_pool = Pool::new(
        Config::from_url(&SESSION_CONFIG.redis_url).unwrap(),
        None,
        None,
        None,
        10,
    )
    .unwrap();

    let redis_conn = redis_pool.connect();

    redis_pool.wait_for_connect().await.unwrap();

    let session_store = RedisStore::new(redis_pool);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_domain(SESSION_CONFIG.domain.clone())
        .with_expiry(Expiry::OnInactivity(Duration::days(30)))
        .with_http_only(true)
        .with_name(SESSION_CONFIG.name.clone())
        .with_private(Key::from(SESSION_CONFIG.key.as_bytes()))
        .with_same_site(SameSite::Strict)
        .with_secure(SESSION_CONFIG.secure);

    let app = axum::Router::new()
        .route("/storage/files/{id}", get(get_storage_file))
        .serve_dioxus_application(ServeConfig::new().unwrap(), App)
        .layer(session_layer);

    let addr = dioxus::cli_config::fullstack_address_or_localhost();
    let listener = TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app.into_make_service()).await.unwrap();

    redis_conn.await.unwrap().unwrap();
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

pub fn use_current_user() -> Resource<Option<UserPresenter>> {
    use_context()
}

#[component]
fn App() -> Element {
    let current_user = use_server_future(async || get_current_user().await.ok().flatten())?;

    use_context_provider(|| current_user);

    rsx! {
        document::Link { rel: "icon", href: FAVICON_ICO }
        document::Link { rel: "stylesheet", href: STYLE_CSS }

        Router::<Routes> {}
    }
}
