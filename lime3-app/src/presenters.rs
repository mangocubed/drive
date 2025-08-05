use serde::{Deserialize, Serialize};
use uuid::Uuid;

use lime3_core::enums::FileVisibility;

#[cfg(feature = "server")]
use lime3_core::server::commands::get_folder_by_id;
#[cfg(feature = "server")]
use lime3_core::server::models::{Folder, User};

#[cfg(feature = "server")]
pub trait AsyncInto<T> {
    fn async_into(&self) -> impl std::future::Future<Output = T>;
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
pub struct UserPresenter {
    id: Uuid,
    pub username: String,
    pub display_name: String,
    pub initials: String,
}

#[cfg(feature = "server")]
impl From<User<'_>> for UserPresenter {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username.to_string(),
            display_name: user.display_name.to_string(),
            initials: user.initials(),
        }
    }
}
