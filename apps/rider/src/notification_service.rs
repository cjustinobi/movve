use crate::model::{MessageResponse, NewMessage, RideStatus, SendMessageRequest, WsMessage};
use crate::repository::RiderRepository;
use common::AppError;
use services::NotificationService as SharedNotificationService;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Clone)]
pub struct NotificationService {
    repository: RiderRepository,
    transport: Arc<SharedNotificationService>,
}

impl NotificationService {
    pub fn new(
        repository: RiderRepository,
        redis_client: redis::Client,
        redis_conn: redis::aio::ConnectionManager,
    ) -> Self {
        let transport = Arc::new(SharedNotificationService::new(redis_client, redis_conn));
        Self {
            repository,
            transport,
        }
    }

    /// Subscribe a specific user to receive notifications
    pub async fn subscribe_user(&self, user_id: Uuid) -> broadcast::Receiver<WsMessage> {
        self.transport.subscribe_user(user_id).await
    }

    /// Unsubscribe a user
    pub async fn unsubscribe_user(&self, user_id: Uuid) {
        self.transport.unsubscribe_user(user_id).await
    }

    /// Send a message and broadcast it
    pub async fn send_message(
        &self,
        sender_id: Uuid,
        sender_role: String,
        req: SendMessageRequest,
    ) -> Result<MessageResponse, AppError> {
        // Validate context type
        use common::models::ChatContextType;
        use std::str::FromStr;
        let _ = ChatContextType::from_str(&req.context_type).map_err(|_| {
            AppError::BadRequest(format!("Invalid context type: {}", req.context_type))
        })?;

        // Get or create conversation
        let conversation = self
            .repository
            .get_or_create_conversation(req.context_type.clone(), req.context_id)
            .await?;

        let new_msg = NewMessage {
            id: Uuid::new_v4(),
            conversation_id: conversation.id,
            sender_id,
            sender_role: sender_role.clone(),
            content: req.content,
        };

        let message = self.repository.create_message(new_msg).await?;
        let response: MessageResponse = message.into();

        // Publish locally and remotely via shared service
        // If context is ride, we need to find the OTHER party.
        if req.context_type == "ride" {
            if let Ok(Some(ride)) = self.repository.get_ride(req.context_id).await {
                let target_id = if sender_role == "rider" {
                    ride.driver_id
                } else {
                    Some(ride.rider_id)
                };

                if let Some(target) = target_id {
                    self.transport
                        .publish_chat_message(target, response.clone())
                        .await
                        .ok();
                }
            }
        }

        Ok(response)
    }

    pub fn broadcast_ride_status(&self, ride_id: Uuid, status: RideStatus, message: String) {
        let msg = WsMessage::RideStatusUpdate(serde_json::json!({
            "ride_id": ride_id,
            "status": status,
            "message": message
        }));

        let transport = self.transport.clone();
        tokio::spawn(async move {
            transport.publish_ride_update(msg).await.ok();
        });
    }

    pub async fn broadcast_ride_status_to_user(
        &self,
        user_id: Uuid,
        ride_id: Uuid,
        status: RideStatus,
        message: String,
    ) {
        let msg = WsMessage::RideStatusUpdate(serde_json::json!({
            "ride_id": ride_id,
            "status": status,
            "message": message
        }));

        self.transport.publish_to_user(user_id, msg).await.ok();
    }

    pub async fn get_messages(
        &self,
        context_type: String,
        context_id: Uuid,
    ) -> Result<Vec<MessageResponse>, AppError> {
        let conversation = self
            .repository
            .get_or_create_conversation(context_type, context_id)
            .await?;

        let messages = self.repository.get_messages(conversation.id).await?;
        let responses = messages.into_iter().map(MessageResponse::from).collect();
        Ok(responses)
    }
}
