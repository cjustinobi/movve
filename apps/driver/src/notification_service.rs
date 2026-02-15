use common::models::WsMessage;
use services::NotificationService as SharedNotificationService;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Clone)]
pub struct NotificationService {
    transport: Arc<SharedNotificationService>,
}

impl NotificationService {
    pub fn new(redis_client: redis::Client, redis_conn: redis::aio::ConnectionManager) -> Self {
        let transport = Arc::new(SharedNotificationService::new(redis_client, redis_conn));
        Self { transport }
    }

    /// Subscribe a specific user to receive notifications
    pub async fn subscribe_user(&self, user_id: Uuid) -> broadcast::Receiver<WsMessage> {
        self.transport.subscribe_user(user_id).await
    }

    /// Unsubscribe a user
    pub async fn unsubscribe_user(&self, user_id: Uuid) {
        self.transport.unsubscribe_user(user_id).await
    }
}
