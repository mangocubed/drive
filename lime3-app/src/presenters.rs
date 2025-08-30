use serde::{Deserialize, Serialize};
use uuid::Uuid;

use lime3_core::enums::FileVisibility;

#[cfg(feature = "server")]
use lime3_core::server::commands::get_folder_by_id;
#[cfg(feature = "server")]
use lime3_core::server::models::{File, Folder, FolderItem, Plan, User};

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
            initials: self.initials(),
            total_space_bytes: total_space.as_u64(),
            used_space_bytes: used_space.as_u64(),
            total_space: total_space.to_string(),
            used_space: used_space.to_string(),
            plan,
            plan_is_cancelable,
        }
    }
}
