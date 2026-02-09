use crate::model::{MessageResponse, NewMessage, RideStatus, SendMessageRequest, WsMessage};
use crate::repository::RiderRepository;
use common::AppError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

type UserConnections = Arc<RwLock<HashMap<Uuid, broadcast::Sender<WsMessage>>>>;

#[derive(Clone)]
pub struct NotificationService {
    repository: RiderRepository,
    // Global broadcast channel for backwards compatibility
    global_tx: broadcast::Sender<WsMessage>,
    // User-specific channels for targeted notifications
    user_channels: UserConnections,
}

impl NotificationService {
    pub fn new(repository: RiderRepository) -> Self {
        let (global_tx, _rx) = broadcast::channel(100);
        let user_channels = Arc::new(RwLock::new(HashMap::new()));
        Self {
            repository,
            global_tx,
            user_channels,
        }
    }

    /// Send a message and broadcast it
    pub async fn send_message(
        &self,
        sender_id: Uuid,
        sender_role: String,
        req: SendMessageRequest,
    ) -> Result<MessageResponse, AppError> {
        // Get or create conversation
        let conversation = self
            .repository
            .get_or_create_conversation(req.context_type, req.context_id)
            .await?;

        let new_msg = NewMessage {
            id: Uuid::new_v4(),
            conversation_id: conversation.id,
            sender_id,
            sender_role,
            content: req.content,
        };

        let message = self.repository.create_message(new_msg).await?;
        let response: MessageResponse = message.into();

        // Broadcast to all connected clients (global)
        let _ = self
            .global_tx
            .send(WsMessage::ChatMessage(response.clone()));

        Ok(response)
    }

    /// Get message history
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

        Ok(messages.into_iter().map(|m| m.into()).collect())
    }

    /// Subscribe to the global message broadcast
    pub fn subscribe(&self) -> broadcast::Receiver<WsMessage> {
        self.global_tx.subscribe()
    }

    /// Subscribe a specific user to receive notifications
    pub async fn subscribe_user(&self, user_id: Uuid) -> broadcast::Receiver<WsMessage> {
        let mut channels = self.user_channels.write().await;

        // Get or create a channel for this user
        let tx = channels.entry(user_id).or_insert_with(|| {
            let (tx, _rx) = broadcast::channel(100);
            tx
        });

        tx.subscribe()
    }

    /// Unsubscribe a user (cleanup when they disconnect)
    pub async fn unsubscribe_user(&self, user_id: Uuid) {
        let mut channels = self.user_channels.write().await;

        // Only remove if there are no active subscribers
        if let Some(tx) = channels.get(&user_id) {
            if tx.receiver_count() == 0 {
                channels.remove(&user_id);
            }
        }
    }

    /// Broadcast ride status update to a specific user
    pub async fn broadcast_ride_status_to_user(
        &self,
        user_id: Uuid,
        ride_id: Uuid,
        status: RideStatus,
        message: String,
    ) {
        let channels = self.user_channels.read().await;

        if let Some(tx) = channels.get(&user_id) {
            let _ = tx.send(WsMessage::RideStatusUpdate {
                ride_id,
                status,
                message,
            });
        }
    }

    /// Broadcast ride status update to all connected clients (fallback)
    pub fn broadcast_ride_status(&self, ride_id: Uuid, status: RideStatus, message: String) {
        let _ = self.global_tx.send(WsMessage::RideStatusUpdate {
            ride_id,
            status,
            message,
        });
    }
}
