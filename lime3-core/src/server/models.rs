use std::borrow::Cow;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use bytesize::ByteSize;
use chrono::{DateTime, NaiveDate, Utc};
use file_format::FileFormat;
use image::metadata::Orientation;
use image::{DynamicImage, ImageDecoder, ImageReader};
use uuid::Uuid;

use crate::enums::FileVisibility;
use crate::server::commands::get_used_storage_by_user;

use super::config::{MEMBERSHIPS_CONFIG, MembershipConfig, STORAGE_CONFIG};
use super::constants::ALLOWED_FILE_FORMATS;

fn verify_password(encrypted_password: &str, password: &str) -> bool {
    let argon2 = Argon2::default();

    let Ok(password_hash) = PasswordHash::new(encrypted_password) else {
        return false;
    };

    argon2.verify_password(password.as_bytes(), &password_hash).is_ok()
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
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl File<'_> {
    pub fn default_path(&self) -> String {
        format!("{}/default.{}", self.directory(), self.format().extension())
    }

    pub fn directory(&self) -> String {
        format!("{}/files/{}", STORAGE_CONFIG.path, self.id)
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
                    dynamic_image.resize_to_fill(width as u32, height as u32, STORAGE_CONFIG.image_ops_filter_type())
                } else {
                    dynamic_image.resize(width as u32, height as u32, STORAGE_CONFIG.image_ops_filter_type())
                };

                dynamic_image.save(variant_path.clone()).unwrap();
            }

            return std::fs::read(variant_path).ok();
        }

        self.read()
    }

    pub fn url(&self) -> String {
        format!("/storage/files/{}", self.id)
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
            "{}/{}x{}{}.{}",
            self.directory(),
            width,
            height,
            if fill { "_fill" } else { "" },
            self.format().extension()
        )
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
}

pub struct Folder<'a> {
    pub id: Uuid,
    pub user_id: Uuid,
    pub parent_folder_id: Option<Uuid>,
    pub name: Cow<'a, str>,
    pub visibility: FileVisibility,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub struct User<'a> {
    pub id: Uuid,
    pub username: Cow<'a, str>,
    pub email: Cow<'a, str>,
    pub encrypted_password: Cow<'a, str>,
    pub display_name: String,
    pub full_name: String,
    pub birthdate: NaiveDate,
    pub language_code: String,
    pub country_alpha2: String,
    pub membership_code: Cow<'a, str>,
    pub has_annual_billing: bool,
    pub disabled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl User<'_> {
    pub async fn available_storage(&self) -> ByteSize {
        self.membership().total_storage - self.used_storage().await
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

    pub fn membership(&self) -> &MembershipConfig {
        MEMBERSHIPS_CONFIG.get(&self.membership_code).unwrap()
    }

    pub async fn used_storage(&self) -> ByteSize {
        get_used_storage_by_user(self).await
    }

    pub fn verify_password(&self, password: &str) -> bool {
        verify_password(&self.encrypted_password, password)
    }
}

pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
    #[allow(dead_code)]
    pub updated_at: Option<DateTime<Utc>>,
}
