use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "server")]
use validator::Validate;

use crate::enums::FileVisibility;

#[cfg(feature = "server")]
use crate::server::constants::REGEX_FILE_NAME;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "server", derive(Validate))]
pub struct FileInput {
    pub parent_folder_id: Option<Uuid>,
    #[cfg_attr(feature = "server", validate(length(min = 1, max = 256, message = "Can't be blank"),
        regex(path = *REGEX_FILE_NAME, message = "Is invalid"),
    ))]
    pub name: String,
    pub content: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "server", derive(Validate))]
pub struct FolderInput {
    pub parent_folder_id: Option<Uuid>,
    #[cfg_attr(feature = "server", validate(length(min = 1, max = 256, message = "Can't be blank"),
        regex(path = *REGEX_FILE_NAME, message = "Is invalid"),
    ))]
    pub name: String,
    pub visibility: FileVisibility,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "server", derive(Validate))]
pub struct RenameInput {
    pub id: Uuid,
    #[cfg_attr(feature = "server", validate(length(min = 1, max = 256, message = "Can't be blank"),
        regex(path = *REGEX_FILE_NAME, message = "Is invalid"),
    ))]
    pub name: String,
}
