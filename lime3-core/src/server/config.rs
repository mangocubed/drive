use std::collections::HashSet;
use std::fs::exists;
use std::sync::LazyLock;

use bytesize::ByteSize;
use figment::Figment;
use figment::providers::{Env, Format, Serialized, Toml};
use image::imageops::FilterType;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

fn extract_from_config<'a, T>(path: &str) -> T
where
    T: Deserialize<'a> + Serialize + Default,
{
    if let Ok(true) = exists("Config.toml") {
        Figment::from(Toml::file("Config.toml")).extract_inner(path).unwrap()
    } else {
        T::default()
    }
}

fn extract_from_env<'a, T>(prefix: &str) -> T
where
    T: Deserialize<'a> + Serialize + Default,
{
    Figment::from(Serialized::defaults(T::default()))
        .merge(Env::prefixed(prefix))
        .extract()
        .unwrap()
}

pub(crate) static BILLING_CONFIG: LazyLock<BillingConfig> = LazyLock::new(|| extract_from_env("BILLING_"));
pub(crate) static DATABASE_CONFIG: LazyLock<DatabaseConfig> = LazyLock::new(|| extract_from_env("DATABASE_"));
pub(crate) static MEMBERSHIPS_CONFIG: LazyLock<MembershipsConfig> =
    LazyLock::new(|| extract_from_config("memberships"));
pub static SESSION_CONFIG: LazyLock<SessionConfig> = LazyLock::new(|| extract_from_env("SESSION_"));
pub(crate) static STORAGE_CONFIG: LazyLock<StorageConfig> = LazyLock::new(|| extract_from_env("STORAGE_"));

#[derive(Deserialize, Serialize)]
pub(crate) struct BillingConfig {
    pub polar_base_url: String,
    pub polar_token: String,
    pub success_base_url: Url,
}

impl Default for BillingConfig {
    fn default() -> Self {
        Self {
            polar_base_url: "https://sandbox-api.polar.sh/v1/".to_owned(),
            polar_token: "".to_owned(),
            success_base_url: "https://example.com/success".parse().unwrap(),
        }
    }
}

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

#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct MembershipConfig {
    pub code: String,
    pub name: String,
    pub description: String,
    pub max_size_per_file: ByteSize,
    pub total_storage: ByteSize,
    pub monthly: Option<MembershipIntervalConfig>,
    pub annual: Option<MembershipIntervalConfig>,
    pub is_restricted: bool,
}

impl MembershipConfig {
    pub fn is_free(&self) -> bool {
        self.monthly.is_none() || self.annual.is_none()
    }
}

#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct MembershipIntervalConfig {
    pub price_cents: u16,
    pub polar_product_id: Uuid,
}

impl MembershipIntervalConfig {
    pub fn price(&self) -> String {
        format!("$ {} USD", self.price_cents as f32 / 100.0)
    }
}

#[derive(Deserialize, Serialize)]
pub(crate) struct MembershipsConfig {
    pub default: String,
    pub options: HashSet<MembershipConfig>,
}

impl Default for MembershipsConfig {
    fn default() -> Self {
        let mut memberships = HashSet::new();

        memberships.insert(MembershipConfig {
            code: "starter".to_owned(),
            name: "Starter".to_owned(),
            description: "A great option for newbies.".to_owned(),
            max_size_per_file: ByteSize::mib(100),
            total_storage: ByteSize::gib(1),
            monthly: None,
            annual: None,
            is_restricted: false,
        });

        Self {
            default: "starter".to_owned(),
            options: memberships,
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
