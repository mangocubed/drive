use std::sync::LazyLock;

use bytesize::ByteSize;
use figment::Figment;
use figment::providers::{Env, Serialized};
use image::imageops::FilterType;
use serde::{Deserialize, Serialize};
use url::Url;

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
pub(crate) static POLAR_CONFIG: LazyLock<PolarConfig> = LazyLock::new(|| extract_from_env("POLAR_"));
pub static SESSION_CONFIG: LazyLock<SessionConfig> = LazyLock::new(|| extract_from_env("SESSION_"));
pub(crate) static STORAGE_CONFIG: LazyLock<StorageConfig> = LazyLock::new(|| extract_from_env("STORAGE_"));
pub(crate) static USERS_CONFIG: LazyLock<UsersConfig> = LazyLock::new(|| extract_from_env("USERS_"));

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
pub(crate) struct PolarConfig {
    pub base_url: Url,
    pub access_token: String,
    pub success_base_url: Url,
}

impl Default for PolarConfig {
    fn default() -> Self {
        Self {
            base_url: "https://sandbox-api.polar.sh/v1/".parse().unwrap(),
            access_token: "".to_owned(),
            success_base_url: "http://127.0.0.1:8080/".parse().unwrap(),
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
    pub image_filter_type: FilterType,
    pub max_size_gib_per_file: u8,
    pub path: String,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            image_filter_type: FilterType::CatmullRom,
            max_size_gib_per_file: 1,
            #[cfg(not(test))]
            path: "./storage".to_owned(),
            #[cfg(test)]
            path: "./storage/tests".to_owned(),
        }
    }
}

impl StorageConfig {
    pub fn max_size_per_file(&self) -> ByteSize {
        ByteSize::gib(self.max_size_gib_per_file as u64)
    }
}

#[derive(Deserialize, Serialize)]
pub(crate) struct UsersConfig {
    pub free_quota_gib: u8,
    pub limit: u8,
}

impl Default for UsersConfig {
    fn default() -> Self {
        Self {
            free_quota_gib: 5,
            limit: 10,
        }
    }
}

impl UsersConfig {
    pub fn free_quota(&self) -> ByteSize {
        ByteSize::gib(self.free_quota_gib as u64)
    }
}
