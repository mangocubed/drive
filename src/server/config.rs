use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use figment::Figment;
use figment::providers::{Env, Serialized};
use serde::{Deserialize, Serialize};

fn extract_from_env<'a, T>(prefix: &str) -> T
where
    T: Deserialize<'a> + Serialize + Default,
{
    Figment::from(Serialized::defaults(T::default()))
        .merge(Env::prefixed(prefix))
        .extract()
        .unwrap()
}

pub static DATABASE_CONFIG: LazyLock<DatabaseConfig> = LazyLock::new(|| extract_from_env("DATABASE_"));

#[derive(Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub max_connections: u8,
    pub url: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        let db_suffix = if cfg!(test) { "test" } else { "dev" };

        Self {
            max_connections: 5,
            url: format!("postgres://lime3:lime3@127.0.0.1:5432/lime3_{db_suffix}"),
        }
    }
}
