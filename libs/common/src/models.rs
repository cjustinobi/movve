use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponse {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub sender_role: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub is_read: bool, // Optional/Default to false if missing from Rider
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum WsMessage {
    #[serde(rename = "ride_update")]
    RideStatusUpdate(serde_json::Value), // Keeping generic for now, or could make it stronger typed
    #[serde(rename = "driver_location")]
    DriverLocationUpdate(serde_json::Value),
    #[serde(rename = "chat_message")]
    ChatMessage(MessageResponse),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedisTargetedMessage {
    pub target_id: Uuid,
    pub message: WsMessage,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChatContextType {
    Ride,
    Order,
    Support,
}

impl std::fmt::Display for ChatContextType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChatContextType::Ride => write!(f, "ride"),
            ChatContextType::Order => write!(f, "order"),
            ChatContextType::Support => write!(f, "support"),
        }
    }
}

impl std::str::FromStr for ChatContextType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ride" => Ok(ChatContextType::Ride),
            "order" => Ok(ChatContextType::Order),
            "support" => Ok(ChatContextType::Support),
            _ => Err(format!("Invalid context type: {}", s)),
        }
    }
}
