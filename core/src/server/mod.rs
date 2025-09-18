use std::sync::LazyLock;

use polar_rs::Polar;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::OnceCell;

pub mod commands;
pub mod config;
pub mod constants;
pub mod models;

use config::{DATABASE_CONFIG, POLAR_CONFIG};

static DB_POOL_CELL: OnceCell<PgPool> = OnceCell::const_new();
static POLAR_CLIENT: LazyLock<Polar> = LazyLock::new(|| {
    Polar::new(POLAR_CONFIG.base_url.clone(), POLAR_CONFIG.access_token.clone()).expect("Could not get Polar client.")
});

async fn db_pool<'a>() -> &'a PgPool {
    DB_POOL_CELL
        .get_or_init(|| async {
            PgPoolOptions::new()
                .max_connections(DATABASE_CONFIG.max_connections as u32)
                .connect(&DATABASE_CONFIG.url)
                .await
                .expect("Could not create DB pool.")
        })
        .await
}
