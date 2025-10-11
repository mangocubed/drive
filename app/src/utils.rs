use crate::presenters::{FolderItemPresenter, FolderPresenter};

pub fn can_be_moved(folder_item: &FolderItemPresenter, target_folder: Option<&FolderPresenter>) -> bool {
    if let Some(target) = target_folder {
        folder_item.parent_folder_id != Some(target.id)
            && (folder_item.is_file
                || (folder_item.id != target.id && !target.parent_folders.iter().any(|pf| pf.id == folder_item.id)))
    } else {
        folder_item.parent_folder_id.is_some()
    }
}
