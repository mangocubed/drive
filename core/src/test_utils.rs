use std::borrow::Cow;

use chrono::Utc;
use sdk::auth_client::UserInfo;
use uuid::Uuid;

pub use sdk::test_utils::{fake_auth, fake_birthdate, fake_country_alpha2, fake_email, fake_name, fake_username};

use crate::enums::FileVisibility;
use crate::inputs::{FileInput, FolderInput};
use crate::server::commands::{insert_file, insert_folder, insert_or_update_user, insert_session};
use crate::server::models::{File, Folder, Session, User};

pub async fn insert_test_file<'a>(user: Option<&User<'_>>) -> File<'a> {
    let user = if let Some(user) = user {
        user
    } else {
        &insert_test_user().await
    };

    let input = FileInput {
        parent_folder_id: None,
        name: fake_name() + ".jpg",
        content: vec![0xFF, 0xD8, 0xFF],
    };

    insert_file(user, &input).await.expect("Could not insert folder")
}

pub async fn insert_test_files<'a>(count: u8, user: Option<&User<'_>>) -> Vec<File<'a>> {
    let user = if let Some(user) = user {
        user
    } else {
        &insert_test_user().await
    };

    let mut files = Vec::new();

    for _ in 0..count {
        let file = insert_test_file(Some(user)).await;
        files.push(file);
    }

    files
}

pub async fn insert_test_folder<'a>(user: Option<&User<'_>>, parent_folder: Option<&Folder<'_>>) -> Folder<'a> {
    let user = if let Some(user) = user {
        user
    } else {
        &insert_test_user().await
    };
    let parent_folder_id = parent_folder.map(|folder| folder.id);

    let input = FolderInput {
        parent_folder_id,
        name: fake_name(),
        visibility: FileVisibility::Private,
    };

    insert_folder(user, &input).await.expect("Could not insert folder")
}

pub async fn insert_test_folders<'a>(
    count: u8,
    user: Option<&User<'_>>,
    parent_folder: Option<&Folder<'_>>,
) -> Vec<Folder<'a>> {
    let user = if let Some(user) = user {
        user
    } else {
        &insert_test_user().await
    };
    let mut folders = Vec::new();

    for _ in 0..count {
        folders.push(insert_test_folder(Some(user), parent_folder).await);
    }

    folders
}

pub async fn insert_test_session<'a>() -> Session<'a> {
    let user = insert_test_user().await;
    let auth = fake_auth();

    insert_session(&user, &auth).await.expect("Could not insert session")
}

pub async fn insert_test_user<'a>() -> User<'a> {
    let user_info = UserInfo {
        id: Uuid::new_v4(),
        username: Cow::Owned(fake_username()),
        email: Cow::Owned(fake_email()),
        display_name: Cow::Owned(fake_name()),
        initials: Cow::Borrowed("AA"),
        full_name: Cow::Owned(fake_name()),
        birthdate: fake_birthdate(),
        language_code: Cow::Borrowed("en"),
        country_alpha2: Cow::Owned(fake_country_alpha2()),
        created_at: Utc::now(),
        updated_at: None,
    };

    insert_or_update_user(&user_info).await.expect("Could not insert user")
}
