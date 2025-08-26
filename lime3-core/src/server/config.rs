use std::ops::RangeInclusive;
use std::sync::LazyLock;

use bytesize::{ByteSize, GIB};
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
pub static PRICING_CONFIG: LazyLock<PricingConfig> = LazyLock::new(|| extract_from_env("PRICING_"));
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

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct PricingConfig {
    pub default_quota: ByteSize,
    pub free_quota: ByteSize,
    pub max_quota: ByteSize,
}

impl Default for PricingConfig {
    fn default() -> Self {
        Self {
            default_quota: ByteSize::gib(1),
            free_quota: ByteSize::gib(1),
            max_quota: ByteSize::gib(10),
        }
    }
}

impl PricingConfig {
    pub fn free_quota_gib(&self) -> u8 {
        (self.free_quota.as_u64() / GIB).try_into().unwrap()
    }

    pub fn max_quota_gib(&self) -> u8 {
        (self.max_quota.as_u64() / GIB).try_into().unwrap()
    }

    pub fn quota_range(&self) -> RangeInclusive<ByteSize> {
        self.free_quota..=self.max_quota
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
    pub max_size_per_file: ByteSize,
    pub path: String,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            image_filter_type: FilterType::CatmullRom,
            max_size_per_file: ByteSize::mib(100),
            #[cfg(not(test))]
            path: "./storage".to_owned(),
            #[cfg(test)]
            path: "./storage/tests".to_owned(),
        }
    }
}
