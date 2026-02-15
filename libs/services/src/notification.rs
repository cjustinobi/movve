use common::models::{MessageResponse, RedisTargetedMessage, WsMessage};
use futures_util::StreamExt;
use redis::AsyncCommands;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

type UserConnections = Arc<RwLock<HashMap<Uuid, broadcast::Sender<WsMessage>>>>;

#[derive(Clone)]
pub struct NotificationService {
    user_channels: UserConnections,
    redis_conn: redis::aio::ConnectionManager,
}

impl NotificationService {
    pub fn new(redis_client: redis::Client, redis_conn: redis::aio::ConnectionManager) -> Self {
        // Explicitly type HashMap to avoid inference errors
        let user_channels: UserConnections = Arc::new(RwLock::new(HashMap::new()));
        let user_channels_clone = user_channels.clone();

        // Spawn Redis subscriber
        tokio::spawn(async move {
            // Use get_async_pubsub instead of into_pubsub (deprecated)
            let mut pubsub = match redis_client.get_async_pubsub().await {
                Ok(conn) => conn,
                Err(e) => {
                    tracing::error!("Failed to get async pubsub: {}", e);
                    return;
                }
            };

            // Subscribe to targeted messages channel
            if let Err(e) = pubsub.subscribe("targeted:messages").await {
                tracing::error!("Failed to subscribe to targeted:messages: {}", e);
                return;
            }
            if let Err(e) = pubsub.subscribe("chat:messages").await {
                tracing::warn!(
                    "Unsubscribing/Ignoring legacy chat:messages channel in new service"
                );
            }
            if let Err(e) = pubsub.subscribe("ride:updates").await {
                tracing::error!("Failed to subscribe to ride:updates: {}", e);
                return;
            }

            let mut stream = pubsub.on_message();

            while let Some(msg) = stream.next().await {
                let channel = msg.get_channel_name().to_string();
                let payload: String = match msg.get_payload() {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::error!("Failed to get payload from Redis message: {}", e);
                        continue;
                    }
                };

                if channel == "targeted:messages" {
                    match serde_json::from_str::<RedisTargetedMessage>(&payload) {
                        Ok(redis_msg) => {
                            let channels = user_channels_clone.read().await;
                            if let Some(tx) = channels.get(&redis_msg.target_id) {
                                let _ = tx.send(redis_msg.message);
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                "Failed to deserialize targeted message from Redis: {}",
                                e
                            );
                        }
                    }
                } else if channel == "ride:updates" {
                    match serde_json::from_str::<WsMessage>(&payload) {
                        Ok(msg) => {
                            // Broadcast to ALL connected users (Global Broadcast)
                            let channels = user_channels_clone.read().await;
                            for tx in channels.values() {
                                let _ = tx.send(msg.clone());
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to deserialize ride update from Redis: {}", e);
                        }
                    }
                }
            }
        });

        Self {
            user_channels,
            redis_conn,
        }
    }

    pub async fn subscribe_user(&self, user_id: Uuid) -> broadcast::Receiver<WsMessage> {
        let mut channels = self.user_channels.write().await;
        let tx = channels.entry(user_id).or_insert_with(|| {
            let (tx, _rx) = broadcast::channel(100);
            tx
        });
        tx.subscribe()
    }

    pub async fn unsubscribe_user(&self, user_id: Uuid) {
        let mut channels = self.user_channels.write().await;
        if let Some(tx) = channels.get(&user_id) {
            if tx.receiver_count() == 0 {
                channels.remove(&user_id);
            }
        }
    }

    pub async fn publish_to_user(
        &self,
        target_id: Uuid,
        msg: WsMessage,
    ) -> Result<(), anyhow::Error> {
        let redis_msg = RedisTargetedMessage {
            target_id,
            message: msg.clone(),
        };

        let payload = serde_json::to_string(&redis_msg)?;
        let mut conn = self.redis_conn.clone();

        let _: () = conn.publish("targeted:messages", payload).await?;

        // Also send locally
        let channels = self.user_channels.read().await;
        if let Some(tx) = channels.get(&target_id) {
            let _ = tx.send(msg);
        }

        Ok(())
    }

    pub async fn publish_chat_message(
        &self,
        target_id: Uuid,
        message: MessageResponse,
    ) -> Result<(), anyhow::Error> {
        self.publish_to_user(target_id, WsMessage::ChatMessage(message))
            .await
    }

    pub async fn publish_ride_update(&self, msg: WsMessage) -> Result<(), anyhow::Error> {
        let payload = serde_json::to_string(&msg)?;
        let mut conn = self.redis_conn.clone();

        let _: () = conn.publish("ride:updates", payload).await?;

        // Broadcast locally to all users
        let channels = self.user_channels.read().await;
        for tx in channels.values() {
            let _ = tx.send(msg.clone());
        }

        Ok(())
    }
}
