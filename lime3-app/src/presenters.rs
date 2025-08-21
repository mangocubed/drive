use serde::{Deserialize, Serialize};
use uuid::Uuid;

use lime3_core::enums::FileVisibility;

#[cfg(feature = "server")]
use lime3_core::server::commands::get_folder_by_id;
#[cfg(feature = "server")]
use lime3_core::server::config::MembershipConfig;
#[cfg(feature = "server")]
use lime3_core::server::models::{File, Folder, FolderItem, User};

#[cfg(feature = "server")]
pub trait AsyncInto<T> {
    fn async_into(&self) -> impl std::future::Future<Output = T>;
}

#[derive(Clone, Deserialize, PartialEq, Serialize)]
pub struct FilePresenter {
    pub id: Uuid,
    pub name: String,
    pub visibility: FileVisibility,
    pub parent_folders: Vec<(Uuid, String)>,
    pub url: String,
    pub preview_url: String,
}

#[cfg(feature = "server")]
impl AsyncInto<FilePresenter> for File<'_> {
    async fn async_into(&self) -> FilePresenter {
        let mut parent_folders = Vec::new();
        let mut parent_folder_id = self.parent_folder_id;

        while let Some(id) = parent_folder_id {
            let parent_folder = get_folder_by_id(id, None).await.unwrap();

            parent_folders.push((parent_folder.id, parent_folder.name.to_string()));

            parent_folder_id = parent_folder.parent_folder_id;
        }

        parent_folders.reverse();

        FilePresenter {
            id: self.id,
            name: self.name.to_string(),
            visibility: self.visibility,
            parent_folders,
            url: self.url(),
            preview_url: self.preview_url(),
        }
    }
}

#[derive(Clone, Deserialize, PartialEq, Serialize)]
pub struct FolderItemPresenter {
    pub id: Uuid,
    pub is_file: bool,
    pub name: String,
    pub visibility: FileVisibility,
    pub preview_url: Option<String>,
}

#[cfg(feature = "server")]
impl AsyncInto<FolderItemPresenter> for FolderItem<'_> {
    async fn async_into(&self) -> FolderItemPresenter {
        let mut parent_folders = Vec::new();
        let mut parent_folder_id = self.parent_folder_id;

        while let Some(id) = parent_folder_id {
            let parent_folder = get_folder_by_id(id, None).await.unwrap();

            parent_folders.push((parent_folder.id, parent_folder.name.to_string()));

            parent_folder_id = parent_folder.parent_folder_id;
        }

        parent_folders.reverse();

        FolderItemPresenter {
            id: self.id,
            is_file: self.is_file,
            name: self.name.to_string(),
            visibility: self.visibility,
            preview_url: self.preview_url(),
        }
    }
}

#[derive(Clone, Deserialize, PartialEq, Serialize)]
pub struct FolderPresenter {
    pub id: Uuid,
    pub name: String,
    pub visibility: FileVisibility,
    pub parent_folders: Vec<(Uuid, String)>,
}

#[cfg(feature = "server")]
impl AsyncInto<FolderPresenter> for Folder<'_> {
    async fn async_into(&self) -> FolderPresenter {
        let mut parent_folders = Vec::new();
        let mut parent_folder_id = self.parent_folder_id;

        while let Some(id) = parent_folder_id {
            let parent_folder = get_folder_by_id(id, None).await.unwrap();

            parent_folders.push((parent_folder.id, parent_folder.name.to_string()));

            parent_folder_id = parent_folder.parent_folder_id;
        }

        parent_folders.reverse();

        FolderPresenter {
            id: self.id,
            name: self.name.to_string(),
            visibility: self.visibility,
            parent_folders,
        }
    }
}

#[derive(Clone, Deserialize, PartialEq, Serialize)]
pub struct MembershipPresenter {
    pub code: String,
    pub name: String,
    pub description: String,
    pub max_size_per_file: String,
    pub total_storage: String,
    pub monthly_price: Option<String>,
    pub annual_price: Option<String>,
    pub is_free: bool,
}

#[cfg(feature = "server")]
impl From<&MembershipConfig> for MembershipPresenter {
    fn from(membership: &MembershipConfig) -> Self {
        Self {
            code: membership.code.clone(),
            name: membership.name.clone(),
            description: membership.description.clone(),
            max_size_per_file: membership.max_size_per_file.to_string(),
            total_storage: membership.total_storage.to_string(),
            monthly_price: membership.monthly.as_ref().map(|monthly| monthly.price()),
            annual_price: membership.annual.as_ref().map(|annual| annual.price()),
            is_free: membership.is_free(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct UserPresenter {
    id: Uuid,
    pub username: String,
    pub display_name: String,
    pub initials: String,
    pub total_storage_bytes: u64,
    pub used_storage_bytes: u64,
    pub total_storage: String,
    pub used_storage: String,
    pub membership: MembershipPresenter,
    pub membership_is_annual: bool,
}

#[cfg(feature = "server")]
impl AsyncInto<UserPresenter> for User<'_> {
    async fn async_into(&self) -> UserPresenter {
        let total_storage = self.membership().total_storage;
        let used_storage = self.used_storage().await;

        UserPresenter {
            id: self.id,
            username: self.username.to_string(),
            display_name: self.display_name.to_string(),
            initials: self.initials(),
            total_storage_bytes: total_storage.as_u64(),
            used_storage_bytes: used_storage.as_u64(),
            total_storage: total_storage.to_string(),
            used_storage: used_storage.to_string(),
            membership: self.membership().into(),
            membership_is_annual: self.membership_is_annual,
        }
    }
}
