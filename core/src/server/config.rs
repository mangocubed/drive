use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use bytesize::ByteSize;
use figment::Figment;
use figment::providers::{Env, Serialized};
use image::imageops::FilterType;
use serde::{Deserialize, Serialize};
use url::Url;

pub fn extract_from_env<'a, T>(prefix: &str) -> T
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
            url: format!("postgres://mango3:mango3@127.0.0.1:5432/drive_{db_suffix}"),
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
pub(crate) struct StorageConfig {
    pub image_filter_type: FilterType,
    max_size_gib_per_file: u8,
    path: String,
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

    pub fn path(&self) -> PathBuf {
        let storage_path = Path::new(&self.path);

        if !storage_path.exists() {
            std::fs::create_dir_all(storage_path).expect("Could not create storage directory");
        }

        storage_path.into()
    }
}

#[derive(Deserialize, Serialize)]
pub(crate) struct UsersConfig {
    pub access_token_length: u8,
    pub free_quota_gib: u8,
    pub limit: u8,
}

impl Default for UsersConfig {
    fn default() -> Self {
        Self {
            access_token_length: 32,
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
