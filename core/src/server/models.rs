use std::borrow::Cow;

use bytesize::ByteSize;
use chrono::{DateTime, NaiveDate, Utc};
use file_format::FileFormat;
use image::metadata::Orientation;
use image::{DynamicImage, ImageDecoder, ImageReader};
use serde::Serialize;
use uuid::Uuid;

use crate::enums::FileVisibility;
use crate::server::commands::{
    folder_is_trashed, get_folder_by_id, get_plan_by_id, get_used_space_by_user, get_user_by_id, verify_password,
};
use crate::server::config::USERS_CONFIG;

use super::config::STORAGE_CONFIG;
use super::constants::ALLOWED_FILE_FORMATS;

pub struct AccessToken<'a> {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: Cow<'a, str>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub struct File<'a> {
    pub id: Uuid,
    pub user_id: Uuid,
    pub parent_folder_id: Option<Uuid>,
    pub name: Cow<'a, str>,
    pub visibility: FileVisibility,
    pub media_type: Cow<'a, str>,
    pub byte_size: i64,
    pub md5_checksum: Cow<'a, str>,
    pub trashed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl File<'_> {
    pub fn cache_directory(&self) -> String {
        format!("{}/cache/files", STORAGE_CONFIG.path)
    }

    pub fn default_path(&self) -> String {
        format!("{}/{}.{}", self.directory(), self.id, self.format().extension())
    }

    pub fn directory(&self) -> String {
        format!("{}/files", STORAGE_CONFIG.path)
    }

    pub fn format(&self) -> &FileFormat {
        ALLOWED_FILE_FORMATS
            .iter()
            .find(|file_format| file_format.media_type() == self.media_type)
            .unwrap_or(&FileFormat::ArbitraryBinaryData)
    }

    pub fn name_without_extension(&self) -> &str {
        self.name.split('.').collect::<Vec<&str>>()[0]
    }

    pub async fn parent_folder(&self) -> Option<Folder<'_>> {
        if let Some(parent_folder_id) = self.parent_folder_id {
            Some(
                get_folder_by_id(parent_folder_id, None)
                    .await
                    .expect("Could not get parent folder"),
            )
        } else {
            None
        }
    }

    pub fn preview_url(&self) -> String {
        format!("{}?width=800&height=800", self.url())
    }

    pub fn read(&self) -> Option<Vec<u8>> {
        std::fs::read(self.default_path()).ok()
    }

    pub fn read_variant(&self, width: Option<u16>, height: Option<u16>, fill: Option<bool>) -> Option<Vec<u8>> {
        if let Some(width) = width
            && let Some(height) = height
        {
            let fill = fill.unwrap_or(false);

            let variant_path = self.variant_path(width, height, fill);

            if !std::path::Path::new(&variant_path).exists() {
                let mut image_decoder = ImageReader::open(self.default_path())
                    .expect("Could not get image")
                    .into_decoder()
                    .expect("Could not convert image into decoder");
                let orientation = image_decoder.orientation().unwrap_or(Orientation::NoTransforms);
                let mut dynamic_image = DynamicImage::from_decoder(image_decoder).expect("Could not get dynamic image");

                dynamic_image.apply_orientation(orientation);

                dynamic_image = if fill {
                    dynamic_image.resize_to_fill(width as u32, height as u32, STORAGE_CONFIG.image_filter_type)
                } else {
                    dynamic_image.resize(width as u32, height as u32, STORAGE_CONFIG.image_filter_type)
                };

                let _ = std::fs::create_dir_all(self.cache_directory());

                dynamic_image.save(variant_path.clone()).unwrap();
            }

            return std::fs::read(variant_path).ok();
        }

        self.read()
    }

    pub fn url(&self) -> String {
        format!("/storage/files/{}", self.id)
    }

    pub async fn user(&self) -> User<'_> {
        get_user_by_id(self.user_id).await.expect("Could not get user")
    }

    pub fn variant_filename(&self, width: Option<u16>, height: Option<u16>, fill: Option<bool>) -> String {
        if let Some(width) = width
            && let Some(height) = height
        {
            let fill = fill.map(|f| if f { "_fill" } else { "" }).unwrap_or_default();

            return format!(
                "{}_{}x{}{}{}",
                self.name_without_extension(),
                width,
                height,
                fill,
                self.format().extension()
            );
        }

        self.name.to_string()
    }

    pub fn variant_path(&self, width: u16, height: u16, fill: bool) -> String {
        format!(
            "{}/{}_{}x{}{}.{}",
            self.cache_directory(),
            self.id,
            width,
            height,
            if fill { "_fill" } else { "" },
            self.format().extension()
        )
    }
}

impl<'a> From<&FolderItem<'a>> for File<'a> {
    fn from(item: &FolderItem<'a>) -> Self {
        Self {
            id: item.id,
            user_id: item.user_id,
            parent_folder_id: item.parent_folder_id,
            name: item.name.clone(),
            visibility: item.visibility,
            media_type: Cow::Borrowed(""),
            byte_size: 0,
            md5_checksum: Cow::Borrowed(""),
            trashed_at: None,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

pub struct Folder<'a> {
    pub id: Uuid,
    pub user_id: Uuid,
    pub parent_folder_id: Option<Uuid>,
    pub name: Cow<'a, str>,
    pub visibility: FileVisibility,
    pub trashed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Folder<'_> {
    pub async fn is_trashed(&self) -> bool {
        folder_is_trashed(self).await
    }

    pub async fn parent_folder(&self) -> Option<Folder<'_>> {
        if let Some(parent_folder_id) = self.parent_folder_id {
            Some(
                get_folder_by_id(parent_folder_id, None)
                    .await
                    .expect("Could not get parent folder"),
            )
        } else {
            None
        }
    }
}

impl<'a> From<&FolderItem<'a>> for Folder<'a> {
    fn from(item: &FolderItem<'a>) -> Self {
        Self {
            id: item.id,
            user_id: item.user_id,
            parent_folder_id: item.parent_folder_id,
            name: item.name.clone(),
            visibility: item.visibility,
            trashed_at: None,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

pub struct FolderItem<'a> {
    pub id: Uuid,
    pub is_file: bool,
    pub user_id: Uuid,
    pub parent_folder_id: Option<Uuid>,
    pub name: Cow<'a, str>,
    pub visibility: FileVisibility,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl FolderItem<'_> {
    pub fn preview_url(&self) -> Option<String> {
        if self.is_file {
            Some(format!("/storage/files/{}?width=200&height=200", self.id))
        } else {
            None
        }
    }

    pub fn url(&self) -> Option<String> {
        if self.is_file {
            Some(format!("/storage/files/{}", self.id))
        } else {
            None
        }
    }
}

#[derive(Serialize)]
pub struct Plan<'a> {
    pub id: Uuid,
    pub name: Cow<'a, str>,
    pub description: Cow<'a, str>,
    pub quota_gib: i16,
    pub monthly_price_cents: i16,
    pub yearly_price_cents: i16,
    pub polar_monthly_product_id: Uuid,
    pub polar_yearly_product_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Plan<'_> {
    pub fn quota(&self) -> ByteSize {
        ByteSize::gib(self.quota_gib as u64)
    }

    pub fn monthly_price(&self) -> String {
        format!("$ {:.2} USD", self.monthly_price_cents as f32 / 100.0)
    }

    pub fn yearly_price(&self) -> String {
        format!("$ {:.2} USD", self.yearly_price_cents as f32 / 100.0)
    }
}

#[derive(Clone)]
pub struct User<'a> {
    pub id: Uuid,
    pub plan_id: Option<Uuid>,
    pub username: Cow<'a, str>,
    pub email: Cow<'a, str>,
    pub encrypted_password: Cow<'a, str>,
    pub display_name: Cow<'a, str>,
    pub full_name: Cow<'a, str>,
    pub birthdate: NaiveDate,
    pub language_code: Cow<'a, str>,
    pub country_alpha2: Cow<'a, str>,
    pub polar_subscription_id: Option<Uuid>,
    pub plan_expires_at: Option<DateTime<Utc>>,
    pub disabled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl User<'_> {
    pub async fn available_space(&self) -> ByteSize {
        self.total_space().await - self.used_space().await
    }

    pub fn initials(&self) -> String {
        self.display_name
            .split_whitespace()
            .filter_map(|word| word.chars().next())
            .collect::<String>()
            .to_uppercase()
    }

    #[allow(dead_code)]
    pub fn is_disabled(&self) -> bool {
        self.disabled_at.is_some()
    }

    pub async fn plan(&self) -> Option<Plan<'_>> {
        if let Some(plan_id) = self.plan_id
            && self.plan_expires_at.is_some_and(|expires_at| expires_at > Utc::now())
        {
            Some(get_plan_by_id(plan_id).await.expect("Could not get plan"))
        } else {
            None
        }
    }

    pub async fn plan_is_cancellable(&self) -> bool {
        self.polar_subscription_id.is_some() && self.used_space().await <= USERS_CONFIG.free_quota()
    }

    pub async fn total_space(&self) -> ByteSize {
        if let Some(plan) = self.plan().await {
            plan.quota()
        } else {
            USERS_CONFIG.free_quota()
        }
    }

    pub async fn used_space(&self) -> ByteSize {
        get_used_space_by_user(self).await
    }

    pub fn verify_password(&self, password: &str) -> bool {
        verify_password(&self.encrypted_password, password)
    }
}
