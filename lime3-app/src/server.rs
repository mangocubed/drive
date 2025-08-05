use tower_sessions::Session;
use uuid::Uuid;

use lime3_core::server::commands::get_user_session_by_id;
use lime3_core::server::models::UserSession;

const SESSION_KEY_USER_SESSION_ID: &str = "user_session_id";

pub trait SessionTrait {
    fn user_session(&self) -> impl Future<Output = anyhow::Result<UserSession>>;

    fn user_session_id(&self) -> impl Future<Output = anyhow::Result<Uuid>>;

    fn remove_user_session(&self) -> impl Future<Output = Result<(), impl std::error::Error>>;

    fn set_user_session(&self, value: UserSession) -> impl Future<Output = Result<(), impl std::error::Error>>;
}

impl SessionTrait for Session {
    async fn user_session(&self) -> anyhow::Result<UserSession> {
        let user_session_id = self.user_session_id().await?;

        Ok(get_user_session_by_id(user_session_id).await?)
    }

    async fn user_session_id(&self) -> anyhow::Result<Uuid> {
        self.get::<Uuid>(SESSION_KEY_USER_SESSION_ID)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Could not get User Session ID"))
    }

    async fn remove_user_session(&self) -> Result<(), impl std::error::Error> {
        self.remove::<Uuid>(SESSION_KEY_USER_SESSION_ID).await.map(|_| ())
    }

    async fn set_user_session(&self, value: UserSession) -> Result<(), impl std::error::Error> {
        self.insert(SESSION_KEY_USER_SESSION_ID, value.id).await
    }
}
