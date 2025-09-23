use std::collections::HashMap;

use dioxus::signals::GlobalSignal;

use crate::presenters::FolderItemPresenter;

pub static LOADER_UNITS: GlobalSignal<HashMap<String, bool>> = GlobalSignal::new(HashMap::new);
pub static MOVE_FOLDER_ITEM: GlobalSignal<Option<FolderItemPresenter>> = GlobalSignal::new(|| None);
