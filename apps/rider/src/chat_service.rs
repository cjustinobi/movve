use crate::model::{MessageResponse, NewMessage, SendMessageRequest, WsMessage};
use crate::repository::RiderRepository;
use common::AppError;
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Clone)]
pub struct ChatService {
    repository: RiderRepository,
    tx: broadcast::Sender<WsMessage>,
}

impl ChatService {
    pub fn new(repository: RiderRepository) -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self { repository, tx }
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

        // Broadcast to all connected clients
        let _ = self.tx.send(WsMessage::ChatMessage(response.clone()));

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

    /// Subscribe to the message broadcast
    pub fn subscribe(&self) -> broadcast::Receiver<WsMessage> {
        self.tx.subscribe()
    }
}
