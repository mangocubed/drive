use dioxus::signals::GlobalSignal;

use crate::presenters::FolderItemPresenter;

pub static MOVE_FOLDER_ITEM: GlobalSignal<Option<FolderItemPresenter>> = GlobalSignal::new(|| None);
