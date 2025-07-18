use serde::{Deserialize, Serialize};
use crate::models::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ApiResponse<T> {
    Success(T),
    Error { message: String, code: Option<String> },
    Stream { chunk: String, is_complete: bool },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub per_page: u32,
    pub total: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

// Request types
#[derive(Debug, Deserialize)]
pub struct CreateConversationRequest {
    pub conversation_type: ConversationType,
    pub title: Option<String>,
    pub participants: Vec<String>, // participant IDs
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub conversation_id: String,
    pub content: String,
    pub message_type: MessageType,
    pub reply_to: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct GetMessagesRequest {
    pub conversation_id: String,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub before: Option<i64>, // timestamp
    pub after: Option<i64>,  // timestamp
}

// Response types
#[derive(Debug, Serialize)]
pub struct ConversationWithParticipants {
    #[serde(flatten)]
    pub conversation: Conversation,
    pub participants: Vec<Participant>,
    pub unread_count: u32,
}

#[derive(Debug, Serialize)]
pub struct MessageWithSender {
    #[serde(flatten)]
    pub message: Message,
    pub sender: Participant,
}

#[derive(Debug, Serialize)]
pub struct StreamingMessageEvent {
    pub message_id: String,
    pub conversation_id: String,
    pub chunk: String,
    pub chunk_index: i32,
    pub is_complete: bool,
}