use tower_sessions::Session;
use uuid::Uuid;

use super::commands::get_user_session_by_id;
use super::constants::SESSION_KEY_USER_SESSION_ID;
use super::models::UserSession;

pub trait SessionTrait {
    fn user_session(&self) -> impl Future<Output = anyhow::Result<UserSession>>;

    fn user_session_id(&self) -> impl Future<Output = anyhow::Result<Uuid>>;

    fn remove_user_session(&self) -> impl Future<Output = Result<(), impl std::error::Error>>;

    fn set_user_session(&self, value: UserSession) -> impl Future<Output = Result<(), impl std::error::Error>>;
}

impl SessionTrait for Session {
    fn user_session(&self) -> impl Future<Output = anyhow::Result<UserSession>> {
        async {
            let user_session_id = self.user_session_id().await?;

            Ok(get_user_session_by_id(user_session_id).await?)
        }
    }

    fn user_session_id(&self) -> impl Future<Output = anyhow::Result<Uuid>> {
        async {
            self.get::<Uuid>(SESSION_KEY_USER_SESSION_ID)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Could not get User Session ID"))
        }
    }

    fn remove_user_session(&self) -> impl Future<Output = Result<(), impl std::error::Error>> {
        async { self.remove::<Uuid>(SESSION_KEY_USER_SESSION_ID).await.map(|_| ()) }
    }

    fn set_user_session(&self, value: UserSession) -> impl Future<Output = Result<(), impl std::error::Error>> {
        self.insert(SESSION_KEY_USER_SESSION_ID, value.id)
    }
}
