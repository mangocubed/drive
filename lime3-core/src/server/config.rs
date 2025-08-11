use std::sync::LazyLock;

use figment::Figment;
use figment::providers::{Env, Serialized};
use image::imageops::FilterType;
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

pub(crate) static DATABASE_CONFIG: LazyLock<DatabaseConfig> = LazyLock::new(|| extract_from_env("DATABASE_"));
pub static SESSION_CONFIG: LazyLock<SessionConfig> = LazyLock::new(|| extract_from_env("SESSION_"));
pub(crate) static STORAGE_CONFIG: LazyLock<StorageConfig> = LazyLock::new(|| extract_from_env("STORAGE_"));

#[derive(Deserialize, Serialize)]
pub(crate) struct DatabaseConfig {
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

#[derive(Deserialize, Serialize)]
pub struct SessionConfig {
    pub domain: String,
    pub key: String,
    pub name: String,
    pub redis_url: String,
    pub secure: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        let db_number = if cfg!(test) { "10" } else { "0" };

        Self {
            domain: "".to_owned(),
            key: "abcdefghijklmnopqrestuvvwxyz0123456789ABCDEFGHIJKLMNOPQRESTUVVWX".to_owned(),
            name: "_lime3_session".to_owned(),
            redis_url: format!("redis://127.0.0.1:6379/{db_number}"),
            secure: false,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub(crate) struct StorageConfig {
    image_ops_filter_type: String,
    pub path: String,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            image_ops_filter_type: "CatmullRom".to_owned(),
            #[cfg(not(test))]
            path: "./storage".to_owned(),
            #[cfg(test)]
            path: "./storage/tests".to_owned(),
        }
    }
}

impl StorageConfig {
    pub(crate) fn image_ops_filter_type(&self) -> FilterType {
        match self.image_ops_filter_type.as_str() {
            "CatmullRom" => FilterType::CatmullRom,
            "Gaussian" => FilterType::Gaussian,
            "Triangle" => FilterType::Triangle,
            "Lanczos3" => FilterType::Lanczos3,
            _ => FilterType::Nearest,
        }
    }
}
