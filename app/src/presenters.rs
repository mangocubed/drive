use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use drive_core::enums::FileVisibility;

#[cfg(feature = "server")]
use drive_core::server::models::{File, Folder, FolderItem, Plan, User};

#[cfg(feature = "server")]
pub trait AsyncInto<T> {
    fn async_into(&self) -> impl std::future::Future<Output = T>;
}

fn file_variant_url(file_url: &Url, width: u16, height: u16, fill: bool) -> Url {
    let mut variant_url = file_url.clone();

    variant_url.set_query(Some(&format!("width={width}&height={height}&fill={fill}")));

    variant_url
}

#[derive(Clone, Deserialize, PartialEq, Serialize)]
pub struct FilePresenter {
    pub id: Uuid,
    pub parent_folder_id: Option<Uuid>,
    pub name: String,
    pub visibility: FileVisibility,
    pub parent_folders: Vec<FolderPresenter>,
    pub url: Url,
}

impl FilePresenter {
    pub fn variant_url(&self, width: u16, height: u16, fill: bool) -> Url {
        file_variant_url(&self.url, width, height, fill)
    }
}

#[cfg(feature = "server")]
impl AsyncInto<FilePresenter> for File<'_> {
    async fn async_into(&self) -> FilePresenter {
        FilePresenter {
            id: self.id,
            parent_folder_id: self.parent_folder_id,
            name: self.name.to_string(),
            visibility: self.visibility,
            parent_folders: futures::future::join_all(
                self.parent_folders().await.iter().map(|folder| folder.async_into()),
            )
            .await,
            url: self.url().await,
        }
    }
}

impl From<FolderItemPresenter> for FilePresenter {
    fn from(folder_item: FolderItemPresenter) -> Self {
        Self {
            id: folder_item.id,
            parent_folder_id: folder_item.parent_folder_id,
            name: folder_item.name.clone(),
            visibility: folder_item.visibility,
            parent_folders: vec![],
            url: folder_item.url.clone().expect("Could not get file url"),
        }
    }
}

#[derive(Clone, Deserialize, PartialEq, Serialize)]
pub struct FolderItemPresenter {
    pub id: Uuid,
    pub parent_folder_id: Option<Uuid>,
    pub is_file: bool,
    pub name: String,
    pub visibility: FileVisibility,
    pub url: Option<Url>,
    pub parent_folders: Vec<FolderPresenter>,
}

impl FolderItemPresenter {
    pub fn variant_url(&self, width: u16, height: u16, fill: bool) -> Option<Url> {
        self.url.as_ref().map(|url| file_variant_url(url, width, height, fill))
    }
}

#[cfg(feature = "server")]
impl AsyncInto<FolderItemPresenter> for FolderItem<'_> {
    async fn async_into(&self) -> FolderItemPresenter {
        FolderItemPresenter {
            id: self.id,
            parent_folder_id: self.parent_folder_id,
            is_file: self.is_file,
            name: self.name.to_string(),
            visibility: self.visibility,
            url: self.url().await,
            parent_folders: futures::future::join_all(
                self.parent_folders().await.iter().map(|folder| folder.async_into()),
            )
            .await,
        }
    }
}

impl From<&FilePresenter> for FolderItemPresenter {
    fn from(file: &FilePresenter) -> Self {
        FolderItemPresenter {
            id: file.id,
            parent_folder_id: file.parent_folder_id,
            is_file: true,
            name: file.name.to_string(),
            visibility: file.visibility,
            url: Some(file.url.clone()),
            parent_folders: file.parent_folders.clone(),
        }
    }
}

impl From<FilePresenter> for FolderItemPresenter {
    fn from(file: FilePresenter) -> Self {
        Self::from(&file)
    }
}

impl From<FolderPresenter> for FolderItemPresenter {
    fn from(folder: FolderPresenter) -> Self {
        Self {
            id: folder.id,
            parent_folder_id: folder.parent_folder_id,
            is_file: false,
            name: folder.name,
            visibility: folder.visibility,
            url: None,
            parent_folders: folder.parent_folders,
        }
    }
}

#[derive(Clone, Deserialize, PartialEq, Serialize)]
pub struct FolderPresenter {
    pub id: Uuid,
    pub parent_folder_id: Option<Uuid>,
    pub name: String,
    pub visibility: FileVisibility,
    pub parent_folders: Vec<FolderPresenter>,
}

#[cfg(feature = "server")]
impl AsyncInto<FolderPresenter> for Folder<'_> {
    async fn async_into(&self) -> FolderPresenter {
        FolderPresenter {
            id: self.id,
            parent_folder_id: self.parent_folder_id,
            name: self.name.to_string(),
            visibility: self.visibility,
            parent_folders: futures::future::join_all(
                self.parent_folders().await.iter().map(|folder| folder.async_into()),
            )
            .await,
        }
    }
}

impl From<&FolderItemPresenter> for FolderPresenter {
    fn from(folder_item: &FolderItemPresenter) -> Self {
        Self {
            id: folder_item.id,
            parent_folder_id: folder_item.parent_folder_id,
            name: folder_item.name.to_string(),
            visibility: folder_item.visibility,
            parent_folders: vec![],
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct PlanPresenter {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub quota: String,
    pub monthly_price: String,
    pub yearly_price: String,
}

#[cfg(feature = "server")]
impl From<&Plan<'_>> for PlanPresenter {
    fn from(plan: &Plan<'_>) -> Self {
        PlanPresenter {
            id: plan.id,
            name: plan.name.to_string(),
            description: plan.description.to_string(),
            quota: plan.quota().to_string(),
            monthly_price: plan.monthly_price(),
            yearly_price: plan.yearly_price(),
        }
    }
}

#[cfg(feature = "server")]
impl From<Plan<'_>> for PlanPresenter {
    fn from(plan: Plan<'_>) -> Self {
        Self::from(&plan)
    }
}

#[derive(Deserialize, Serialize)]
pub struct UserPresenter {
    id: Uuid,
    pub username: String,
    pub display_name: String,
    pub initials: String,
    pub total_space_bytes: u64,
    pub used_space_bytes: u64,
    pub total_space: String,
    pub used_space: String,
    pub plan: Option<PlanPresenter>,
    pub plan_is_cancelable: bool,
}

#[cfg(feature = "server")]
impl AsyncInto<UserPresenter> for User<'_> {
    async fn async_into(&self) -> UserPresenter {
        let (total_space, used_space, plan, plan_is_cancelable) = futures::future::join4(
            self.total_space(),
            self.used_space(),
            async { self.plan().await.map(|plan| plan.into()) },
            self.plan_is_cancellable(),
        )
        .await;

        UserPresenter {
            id: self.id,
            username: self.username.to_string(),
            display_name: self.display_name.to_string(),
            initials: self.initials.to_string(),
            total_space_bytes: total_space.as_u64(),
            used_space_bytes: used_space.as_u64(),
            total_space: total_space.to_string(),
            used_space: used_space.to_string(),
            plan,
            plan_is_cancelable,
        }
    }
}
