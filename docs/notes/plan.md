Here's a comprehensive API framework that handles both use cases with a unified chat interface:

## **1. Backend API Framework**

### Database Schema & Models
```rust
// src/models/mod.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Participant {
    pub id: String,
    pub participant_type: ParticipantType,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub metadata: Option<String>, // JSON for agent config, user preferences, etc.
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum ParticipantType {
    User,
    Agent,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Conversation {
    pub id: String,
    pub conversation_type: ConversationType,
    pub title: Option<String>,
    pub created_by: String, // participant_id
    pub created_at: i64,
    pub updated_at: i64,
    pub last_message_at: Option<i64>,
    pub metadata: Option<String>, // JSON for conversation-specific data
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum ConversationType {
    DirectMessage,
    AgentChat,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConversationParticipant {
    pub conversation_id: String,
    pub participant_id: String,
    pub role: ParticipantRole,
    pub joined_at: i64,
    pub last_read_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum ParticipantRole {
    Member,
    Owner,
    Agent,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub content: String,
    pub message_type: MessageType,
    pub reply_to: Option<String>,
    pub sent_at: i64,
    pub delivered_at: Option<i64>,
    pub read_at: Option<i64>,
    pub metadata: Option<String>, // JSON for message-specific data
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum MessageType {
    Text,
    Image,
    File,
    System,
    Typing,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MessageStream {
    pub id: String,
    pub message_id: String,
    pub chunk_index: i32,
    pub content: String,
    pub is_complete: bool,
    pub created_at: i64,
}
```

### Database Migrations
```sql
-- migrations/001_initial_schema.sql
CREATE TABLE participants (
    id TEXT PRIMARY KEY,
    participant_type TEXT NOT NULL CHECK (participant_type IN ('User', 'Agent')),
    display_name TEXT NOT NULL,
    avatar_url TEXT,
    metadata TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE conversations (
    id TEXT PRIMARY KEY,
    conversation_type TEXT NOT NULL CHECK (conversation_type IN ('DirectMessage', 'AgentChat')),
    title TEXT,
    created_by TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    last_message_at INTEGER,
    metadata TEXT,
    FOREIGN KEY (created_by) REFERENCES participants(id)
);

CREATE TABLE conversation_participants (
    conversation_id TEXT NOT NULL,
    participant_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('Member', 'Owner', 'Agent')),
    joined_at INTEGER NOT NULL,
    last_read_at INTEGER,
    PRIMARY KEY (conversation_id, participant_id),
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (participant_id) REFERENCES participants(id)
);

CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    sender_id TEXT NOT NULL,
    content TEXT NOT NULL,
    message_type TEXT NOT NULL CHECK (message_type IN ('Text', 'Image', 'File', 'System', 'Typing')),
    reply_to TEXT,
    sent_at INTEGER NOT NULL,
    delivered_at INTEGER,
    read_at INTEGER,
    metadata TEXT,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (sender_id) REFERENCES participants(id),
    FOREIGN KEY (reply_to) REFERENCES messages(id)
);

CREATE TABLE message_streams (
    id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL,
    chunk_index INTEGER NOT NULL,
    content TEXT NOT NULL,
    is_complete BOOLEAN NOT NULL DEFAULT FALSE,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (message_id) REFERENCES messages(id)
);

-- Indexes
CREATE INDEX idx_conversations_created_by ON conversations(created_by);
CREATE INDEX idx_conversations_updated_at ON conversations(updated_at);
CREATE INDEX idx_messages_conversation_id ON messages(conversation_id);
CREATE INDEX idx_messages_sent_at ON messages(sent_at);
CREATE INDEX idx_message_streams_message_id ON message_streams(message_id);
CREATE INDEX idx_message_streams_chunk_index ON message_streams(message_id, chunk_index);
```

### API Response Types
```rust
// src/api/types.rs
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
```

### API Services
```rust
// src/api/services.rs
use crate::models::*;
use crate::api::types::*;
use sqlx::SqlitePool;
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct ChatService {
    pool: SqlitePool,
    event_sender: broadcast::Sender<ChatEvent>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ChatEvent {
    MessageReceived { message: MessageWithSender },
    MessageStream { event: StreamingMessageEvent },
    ConversationUpdated { conversation: ConversationWithParticipants },
    ParticipantJoined { conversation_id: String, participant: Participant },
    ParticipantLeft { conversation_id: String, participant_id: String },
    TypingStarted { conversation_id: String, participant_id: String },
    TypingStop { conversation_id: String, participant_id: String },
}

impl ChatService {
    pub fn new(pool: SqlitePool) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        Self { pool, event_sender }
    }

    pub fn subscribe_to_events(&self) -> broadcast::Receiver<ChatEvent> {
        self.event_sender.subscribe()
    }

    pub async fn create_conversation(
        &self,
        request: CreateConversationRequest,
        created_by: String,
    ) -> Result<ConversationWithParticipants, sqlx::Error> {
        let conversation_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp_millis();

        let conversation = Conversation {
            id: conversation_id.clone(),
            conversation_type: request.conversation_type,
            title: request.title,
            created_by: created_by.clone(),
            created_at: now,
            updated_at: now,
            last_message_at: None,
            metadata: request.metadata.map(|m| m.to_string()),
        };

        // Insert conversation
        sqlx::query(
            r#"
            INSERT INTO conversations (id, conversation_type, title, created_by, created_at, updated_at, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&conversation.id)
        .bind(serde_json::to_string(&conversation.conversation_type).unwrap())
        .bind(&conversation.title)
        .bind(&conversation.created_by)
        .bind(conversation.created_at)
        .bind(conversation.updated_at)
        .bind(&conversation.metadata)
        .execute(&self.pool)
        .await?;

        // Add participants
        for participant_id in &request.participants {
            let role = if participant_id == &created_by {
                ParticipantRole::Owner
            } else {
                ParticipantRole::Member
            };

            sqlx::query(
                r#"
                INSERT INTO conversation_participants (conversation_id, participant_id, role, joined_at)
                VALUES (?, ?, ?, ?)
                "#
            )
            .bind(&conversation_id)
            .bind(participant_id)
            .bind(serde_json::to_string(&role).unwrap())
            .bind(now)
            .execute(&self.pool)
            .await?;
        }

        // Also add creator if not in participants list
        if !request.participants.contains(&created_by) {
            sqlx::query(
                r#"
                INSERT INTO conversation_participants (conversation_id, participant_id, role, joined_at)
                VALUES (?, ?, ?, ?)
                "#
            )
            .bind(&conversation_id)
            .bind(&created_by)
            .bind(serde_json::to_string(&ParticipantRole::Owner).unwrap())
            .bind(now)
            .execute(&self.pool)
            .await?;
        }

        let result = self.get_conversation_with_participants(&conversation_id).await?;
        
        // Broadcast event
        let _ = self.event_sender.send(ChatEvent::ConversationUpdated {
            conversation: result.clone(),
        });

        Ok(result)
    }

    pub async fn send_message(
        &self,
        request: SendMessageRequest,
        sender_id: String,
    ) -> Result<MessageWithSender, sqlx::Error> {
        let message_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp_millis();

        let message = Message {
            id: message_id.clone(),
            conversation_id: request.conversation_id.clone(),
            sender_id: sender_id.clone(),
            content: request.content,
            message_type: request.message_type,
            reply_to: request.reply_to,
            sent_at: now,
            delivered_at: Some(now),
            read_at: None,
            metadata: request.metadata.map(|m| m.to_string()),
        };

        // Insert message
        sqlx::query(
            r#"
            INSERT INTO messages (id, conversation_id, sender_id, content, message_type, reply_to, sent_at, delivered_at, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&message.id)
        .bind(&message.conversation_id)
        .bind(&message.sender_id)
        .bind(&message.content)
        .bind(serde_json::to_string(&message.message_type).unwrap())
        .bind(&message.reply_to)
        .bind(message.sent_at)
        .bind(message.delivered_at)
        .bind(&message.metadata)
        .execute(&self.pool)
        .await?;

        // Update conversation last_message_at
        sqlx::query(
            "UPDATE conversations SET last_message_at = ?, updated_at = ? WHERE id = ?"
        )
        .bind(now)
        .bind(now)
        .bind(&message.conversation_id)
        .execute(&self.pool)
        .await?;

        let result = self.get_message_with_sender(&message_id).await?;
        
        // Broadcast event
        let _ = self.event_sender.send(ChatEvent::MessageReceived {
            message: result.clone(),
        });

        Ok(result)
    }

    pub async fn stream_message(
        &self,
        conversation_id: String,
        sender_id: String,
        content_stream: tokio::sync::mpsc::Receiver<String>,
    ) -> Result<String, sqlx::Error> {
        let message_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp_millis();

        // Create initial message
        let message = Message {
            id: message_id.clone(),
            conversation_id: conversation_id.clone(),
            sender_id,
            content: String::new(), // Will be built from streams
            message_type: MessageType::Text,
            reply_to: None,
            sent_at: now,
            delivered_at: Some(now),
            read_at: None,
            metadata: None,
        };

        sqlx::query(
            r#"
            INSERT INTO messages (id, conversation_id, sender_id, content, message_type, sent_at, delivered_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&message.id)
        .bind(&message.conversation_id)
        .bind(&message.sender_id)
        .bind(&message.content)
        .bind(serde_json::to_string(&message.message_type).unwrap())
        .bind(message.sent_at)
        .bind(message.delivered_at)
        .execute(&self.pool)
        .await?;

        // Handle streaming in a separate task
        let pool = self.pool.clone();
        let event_sender = self.event_sender.clone();
        let msg_id = message_id.clone();
        let conv_id = conversation_id.clone();

        tokio::spawn(async move {
            let mut content_stream = content_stream;
            let mut chunk_index = 0;
            let mut full_content = String::new();

            while let Some(chunk) = content_stream.recv().await {
                full_content.push_str(&chunk);
                
                // Store chunk
                let stream_id = Uuid::new_v4().to_string();
                let _ = sqlx::query(
                    r#"
                    INSERT INTO message_streams (id, message_id, chunk_index, content, is_complete, created_at)
                    VALUES (?, ?, ?, ?, ?, ?)
                    "#
                )
                .bind(&stream_id)
                .bind(&msg_id)
                .bind(chunk_index)
                .bind(&chunk)
                .bind(false)
                .bind(chrono::Utc::now().timestamp_millis())
                .execute(&pool)
                .await;

                // Broadcast chunk
                let _ = event_sender.send(ChatEvent::MessageStream {
                    event: StreamingMessageEvent {
                        message_id: msg_id.clone(),
                        conversation_id: conv_id.clone(),
                        chunk: chunk.clone(),
                        chunk_index,
                        is_complete: false,
                    },
                });

                chunk_index += 1;
            }

            // Mark as complete and update message content
            let _ = sqlx::query(
                "UPDATE messages SET content = ? WHERE id = ?"
            )
            .bind(&full_content)
            .bind(&msg_id)
            .execute(&pool)
            .await;

            // Mark last chunk as complete
            let _ = sqlx::query(
                "UPDATE message_streams SET is_complete = TRUE WHERE message_id = ? AND chunk_index = ?"
            )
            .bind(&msg_id)
            .bind(chunk_index - 1)
            .execute(&pool)
            .await;

            // Broadcast completion
            let _ = event_sender.send(ChatEvent::MessageStream {
                event: StreamingMessageEvent {
                    message_id: msg_id,
                    conversation_id: conv_id,
                    chunk: String::new(),
                    chunk_index,
                    is_complete: true,
                },
            });
        });

        Ok(message_id)
    }

    pub async fn get_conversations(
        &self,
        participant_id: &str,
        page: u32,
        per_page: u32,
    ) -> Result<PaginatedResponse<ConversationWithParticipants>, sqlx::Error> {
        let offset = (page - 1) * per_page;

        let conversations = sqlx::query_as::<_, Conversation>(
            r#"
            SELECT c.* FROM conversations c
            JOIN conversation_participants cp ON c.id = cp.conversation_id
            WHERE cp.participant_id = ?
            ORDER BY c.last_message_at DESC NULLS LAST, c.updated_at DESC
            LIMIT ? OFFSET ?
            "#
        )
        .bind(participant_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*) FROM conversations c
            JOIN conversation_participants cp ON c.id = cp.conversation_id
            WHERE cp.participant_id = ?
            "#
        )
        .bind(participant_id)
        .fetch_one(&self.pool)
        .await?
        .0 as u32;

        let mut results = Vec::new();
        for conversation in conversations {
            if let Ok(conv_with_participants) = self.get_conversation_with_participants(&conversation.id).await {
                results.push(conv_with_participants);
            }
        }

        Ok(PaginatedResponse {
            data: results,
            pagination: PaginationInfo {
                page,
                per_page,
                total,
                has_next: offset + per_page < total,
                has_prev: page > 1,
            },
        })
    }

    pub async fn get_messages(
        &self,
        request: GetMessagesRequest,
    ) -> Result<PaginatedResponse<MessageWithSender>, sqlx::Error> {
        let page = request.page.unwrap_or(1);
        let per_page = request.per_page.unwrap_or(50);
        let offset = (page - 1) * per_page;

        let mut query = String::from(
            "SELECT m.* FROM messages m WHERE m.conversation_id = ?"
        );
        let mut params = vec![request.conversation_id.clone()];

        if let Some(before) = request.before {
            query.push_str(" AND m.sent_at < ?");
            params.push(before.to_string());
        }

        if let Some(after) = request.after {
            query.push_str(" AND m.sent_at > ?");
            params.push(after.to_string());
        }

        query.push_str(" ORDER BY m.sent_at DESC LIMIT ? OFFSET ?");
        params.push(per_page.to_string());
        params.push(offset.to_string());

        let messages = sqlx::query_as::<_, Message>(&query)
            .bind(&params[0])
            .bind(params.get(1).unwrap_or(&"".to_string()))
            .bind(params.get(2).unwrap_or(&"".to_string()))
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let total = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM messages WHERE conversation_id = ?"
        )
        .bind(&request.conversation_id)
        .fetch_one(&self.pool)
        .await?
        .0 as u32;

        let mut results = Vec::new();
        for message in messages {
            if let Ok(msg_with_sender) = self.get_message_with_sender(&message.id).await {
                results.push(msg_with_sender);
            }
        }

        Ok(PaginatedResponse {
            data: results,
            pagination: PaginationInfo {
                page,
                per_page,
                total,
                has_next: offset + per_page < total,
                has_prev: page > 1,
            },
        })
    }

    // Helper methods
    async fn get_conversation_with_participants(
        &self,
        conversation_id: &str,
    ) -> Result<ConversationWithParticipants, sqlx::Error> {
        let conversation = sqlx::query_as::<_, Conversation>(
            "SELECT * FROM conversations WHERE id = ?"
        )
        .bind(conversation_id)
        .fetch_one(&self.pool)
        .await?;

        let participants = sqlx::query_as::<_, Participant>(
            r#"
            SELECT p.* FROM participants p
            JOIN conversation_participants cp ON p.id = cp.participant_id
            WHERE cp.conversation_id = ?
            "#
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await?;

        let unread_count = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*) FROM messages m
            LEFT JOIN conversation_participants cp ON m.conversation_id = cp.conversation_id
            WHERE m.conversation_id = ? AND (cp.last_read_at IS NULL OR m.sent_at > cp.last_read_at)
            "#
        )
        .bind(conversation_id)
        .fetch_one(&self.pool)
        .await?
        .0 as u32;

        Ok(ConversationWithParticipants {
            conversation,
            participants,
            unread_count,
        })
    }

    async fn get_message_with_sender(
        &self,
        message_id: &str,
    ) -> Result<MessageWithSender, sqlx::Error> {
        let message = sqlx::query_as::<_, Message>(
            "SELECT * FROM messages WHERE id = ?"
        )
        .bind(message_id)
        .fetch_one(&self.pool)
        .await?;

        let sender = sqlx::query_as::<_, Participant>(
            "SELECT * FROM participants WHERE id = ?"
        )
        .bind(&message.sender_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(MessageWithSender { message, sender })
    }
}
```

### Tauri Commands
```rust
// src/api/commands.rs
use tauri::State;
use crate::api::services::ChatService;
use crate::api::types::*;

#[tauri::command]
pub async fn create_conversation(
    request: CreateConversationRequest,
    created_by: String,
    chat_service: State<'_, ChatService>,
) -> Result<ApiResponse<ConversationWithParticipants>, String> {
    match chat_service.create_conversation(request, created_by).await {
        Ok(conversation) => Ok(ApiResponse::Success(conversation)),
        Err(e) => Ok(ApiResponse::Error {
            message: e.to_string(),
            code: Some("CREATE_CONVERSATION_ERROR".to_string()),
        }),
    }
}

#[tauri::command]
pub async fn send_message(
    request: SendMessageRequest,
    sender_id: String,
    chat_service: State<'_, ChatService>,
) -> Result<ApiResponse<MessageWithSender>, String> {
    match chat_service.send_message(request, sender_id).await {
        Ok(message) => Ok(ApiResponse::Success(message)),
        Err(e) => Ok(ApiResponse::Error {
            message: e.to_string(),
            code: Some("SEND_MESSAGE_ERROR".to_string()),
        }),
    }
}

#[tauri::command]
pub async fn get_conversations(
    participant_id: String,
    page: Option<u32>,
    per_page: Option<u32>,
    chat_service: State<'_, ChatService>,
) -> Result<ApiResponse<PaginatedResponse<ConversationWithParticipants>>, String> {
    let page = page.unwrap_or(1);
    let per_page = per_page.unwrap_or(20);
    
    match chat_service.get_conversations(&participant_id, page, per_page).await {
        Ok(conversations) => Ok(ApiResponse::Success(conversations)),
        Err(e) => Ok(ApiResponse::Error {
            message: e.to_string(),
            code: Some("GET_CONVERSATIONS_ERROR".to_string()),
        }),
    }
}

#[tauri::command]
pub async fn get_messages(
    request: GetMessagesRequest,
    chat_service: State<'_, ChatService>,
) -> Result<ApiResponse<PaginatedResponse<MessageWithSender>>, String> {
    match chat_service.get_messages(request).await {
        Ok(messages) => Ok(ApiResponse::Success(messages)),
        Err(e) => Ok(ApiResponse::Error {
            message: e.to_string(),
            code: Some("GET_MESSAGES_ERROR".to_string()),
        }),
    }
}

#[tauri::command]
pub async fn start_agent_chat(
    agent_id: String,
    user_id: String,
    initial_message: Option<String>,
    chat_service: State<'_, ChatService>,
) -> Result<ApiResponse<ConversationWithParticipants>, String> {
    let request = CreateConversationRequest {
        conversation_type: crate::models::ConversationType::AgentChat,
        title: Some("Agent Chat".to_string()),
        participants: vec![agent_id.clone()],
        metadata: Some(serde_json::json!({
            "agent_id": agent_id,
            "auto_stream": true
        })),
    };

    match chat_service.create_conversation(request, user_id.clone()).await {
        Ok(conversation) => {
            // Send initial message if provided
            if let Some(content) = initial_message {
                let message_request = SendMessageRequest {
                    conversation_id: conversation.conversation.id.clone(),
                    content,
                    message_type: crate::models::MessageType::Text,
                    reply_to: None,
                    metadata: None,
                };
                
                let _ = chat_service.send_message(message_request, user_id).await;
            }
            
            Ok(ApiResponse::Success(conversation))
        }
        Err(e) => Ok(ApiResponse::Error {
            message: e.to_string(),
            code: Some("START_AGENT_CHAT_ERROR".to_string()),
        }),
    }
}

// Event streaming command
#[tauri::command]
pub async fn subscribe_to_chat_events(
    app_handle: tauri::AppHandle,
    chat_service: State<'_, ChatService>,
) -> Result<(), String> {
    let mut receiver = chat_service.subscribe_to_events();
    
    tokio::spawn(async move {
        while let Ok(event) = receiver.recv().await {
            let _ = app_handle.emit_all("chat_event", &event);
        }
    });
    
    Ok(())
}
```

## **2. Frontend API Framework**

### API Types & Client
```typescript
// src/types/api.ts
export interface ApiResponse<T> {
  type: 'Success' | 'Error' | 'Stream';
  data?: T;
  message?: string;
  code?: string;
  chunk?: string;
  is_complete?: boolean;
}

export interface PaginatedResponse<T> {
  data: T[];
  pagination: {
    page: number;
    per_page: number;
    total: number;
    has_next: boolean;
    has_prev: boolean;
  };
}

export enum ParticipantType {
  User = 'User',
  Agent = 'Agent',
}

export enum ConversationType {
  DirectMessage = 'DirectMessage',
  AgentChat = 'AgentChat',
}

export enum MessageType {
  Text = 'Text',
  Image = 'Image',
  File = 'File',
  System = 'System',
  Typing = 'Typing',
}

export interface Participant {
  id: string;
  participant_type: ParticipantType;
  display_name: string;
  avatar_url?: string;
  metadata?: string;
  created_at: number;
  updated_at: number;
}

export interface Conversation {
  id: string;
  conversation_type: ConversationType;
  title?: string;
  created_by: string;
  created_at: number;
  updated_at: number;
  last_message_at?: number;
  metadata?: string;
}

export interface ConversationWithPartic
ipants {
  conversation: Conversation;
  participants: Participant[];
  unread_count: number;
}

export interface Message {
  id: string;
  conversation_id: string;
  sender_id: string;
  content: string;
  message_type: MessageType;
  reply_to?: string;
  sent_at: number;
  delivered_at?: number;
  read_at?: number;
  metadata?: string;
}

export interface MessageWithSender {
  message: Message;
  sender: Participant;
}

export interface ChatEvent {
  type: 'MessageReceived' | 'MessageStream' | 'ConversationUpdated' | 'ParticipantJoined' | 'ParticipantLeft' | 'TypingStarted' | 'TypingStop';
  message?: MessageWithSender;
  event?: {
    message_id: string;
    conversation_id: string;
    chunk: string;
    chunk_index: number;
    is_complete: boolean;
  };
  conversation?: ConversationWithParticipants;
  conversation_id?: string;
  participant?: Participant;
  participant_id?: string;
}
```

### API Client
```typescript
// src/api/client.ts
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { 
  ApiResponse, 
  PaginatedResponse, 
  ConversationWithParticipants, 
  MessageWithSender, 
  ChatEvent,
  ConversationType,
  MessageType
} from '../types/api';

export class ChatAPI {
  private eventCallbacks: Map<string, (event: ChatEvent) => void> = new Map();

  constructor() {
    this.setupEventListeners();
  }

  private async setupEventListeners() {
    await invoke('subscribe_to_chat_events');
    
    await listen<ChatEvent>('chat_event', (event) => {
      this.eventCallbacks.forEach(callback => callback(event.payload));
    });
  }

  public onEvent(id: string, callback: (event: ChatEvent) => void) {
    this.eventCallbacks.set(id, callback);
  }

  public offEvent(id: string) {
    this.eventCallbacks.delete(id);
  }

  async createConversation(
    conversationType: ConversationType,
    participants: string[],
    createdBy: string,
    title?: string,
    metadata?: any
  ): Promise<ConversationWithParticipants> {
    const response = await invoke<ApiResponse<ConversationWithParticipants>>('create_conversation', {
      request: {
        conversation_type: conversationType,
        title,
        participants,
        metadata,
      },
      createdBy,
    });

    if (response.type === 'Success' && response.data) {
      return response.data;
    }

    throw new Error(response.message || 'Failed to create conversation');
  }

  async sendMessage(
    conversationId: string,
    content: string,
    senderId: string,
    messageType: MessageType = MessageType.Text,
    replyTo?: string,
    metadata?: any
  ): Promise<MessageWithSender> {
    const response = await invoke<ApiResponse<MessageWithSender>>('send_message', {
      request: {
        conversation_id: conversationId,
        content,
        message_type: messageType,
        reply_to: replyTo,
        metadata,
      },
      senderId,
    });

    if (response.type === 'Success' && response.data) {
      return response.data;
    }

    throw new Error(response.message || 'Failed to send message');
  }

  async getConversations(
    participantId: string,
    page: number = 1,
    perPage: number = 20
  ): Promise<PaginatedResponse<ConversationWithParticipants>> {
    const response = await invoke<ApiResponse<PaginatedResponse<ConversationWithParticipants>>>('get_conversations', {
      participantId,
      page,
      perPage,
    });

    if (response.type === 'Success' && response.data) {
      return response.data;
    }

    throw new Error(response.message || 'Failed to get conversations');
  }

  async getMessages(
    conversationId: string,
    page: number = 1,
    perPage: number = 50,
    before?: number,
    after?: number
  ): Promise<PaginatedResponse<MessageWithSender>> {
    const response = await invoke<ApiResponse<PaginatedResponse<MessageWithSender>>>('get_messages', {
      request: {
        conversation_id: conversationId,
        page,
        per_page: perPage,
        before,
        after,
      },
    });

    if (response.type === 'Success' && response.data) {
      return response.data;
    }

    throw new Error(response.message || 'Failed to get messages');
  }

  async startAgentChat(
    agentId: string,
    userId: string,
    initialMessage?: string
  ): Promise<ConversationWithParticipants> {
    const response = await invoke<ApiResponse<ConversationWithParticipants>>('start_agent_chat', {
      agentId,
      userId,
      initialMessage,
    });

    if (response.type === 'Success' && response.data) {
      return response.data;
    }

    throw new Error(response.message || 'Failed to start agent chat');
  }
}

export const chatAPI = new ChatAPI();
```

### React Hooks for State Management
```typescript
// src/hooks/useChat.ts
import { useState, useEffect, useCallback } from 'react';
import { chatAPI } from '../api/client';
import { 
  ConversationWithParticipants, 
  MessageWithSender, 
  ChatEvent,
  PaginatedResponse 
} from '../types/api';

interface ChatState {
  conversations: ConversationWithParticipants[];
  currentConversation: ConversationWithParticipants | null;
  messages: MessageWithSender[];
  streamingMessages: Map<string, string>;
  loading: boolean;
  error: string | null;
}

export const useChat = (userId: string) => {
  const [state, setState] = useState<ChatState>({
    conversations: [],
    currentConversation: null,
    messages: [],
    streamingMessages: new Map(),
    loading: false,
    error: null,
  });

  const [pagination, setPagination] = useState({
    page: 1,
    hasNext: false,
    hasPrev: false,
    total: 0,
  });

  // Load conversations
  const loadConversations = useCallback(async () => {
    if (!userId) return;

    setState(prev => ({ ...prev, loading: true, error: null }));

    try {
      const response = await chatAPI.getConversations(userId);
      setState(prev => ({
        ...prev,
        conversations: response.data,
        loading: false,
      }));
    } catch (error) {
      setState(prev => ({
        ...prev,
        error: error instanceof Error ? error.message : 'Failed to load conversations',
        loading: false,
      }));
    }
  }, [userId]);

  // Load messages for a conversation
  const loadMessages = useCallback(async (conversationId: string, page: number = 1) => {
    setState(prev => ({ ...prev, loading: true, error: null }));

    try {
      const response = await chatAPI.getMessages(conversationId, page);
      setState(prev => ({
        ...prev,
        messages: page === 1 ? response.data : [...prev.messages, ...response.data],
        loading: false,
      }));
      setPagination({
        page: response.pagination.page,
        hasNext: response.pagination.has_next,
        hasPrev: response.pagination.has_prev,
        total: response.pagination.total,
      });
    } catch (error) {
      setState(prev => ({
        ...prev,
        error: error instanceof Error ? error.message : 'Failed to load messages',
        loading: false,
      }));
    }
  }, []);

  // Send message
  const sendMessage = useCallback(async (content: string, replyTo?: string) => {
    if (!state.currentConversation) return;

    try {
      await chatAPI.sendMessage(
        state.currentConversation.conversation.id,
        content,
        userId,
        MessageType.Text,
        replyTo
      );
      // Message will be added via event listener
    } catch (error) {
      setState(prev => ({
        ...prev,
        error: error instanceof Error ? error.message : 'Failed to send message',
      }));
    }
  }, [state.currentConversation, userId]);

  // Select conversation
  const selectConversation = useCallback((conversation: ConversationWithParticipants) => {
    setState(prev => ({ ...prev, currentConversation: conversation, messages: [] }));
    loadMessages(conversation.conversation.id);
  }, [loadMessages]);

  // Handle chat events
  useEffect(() => {
    const handleChatEvent = (event: ChatEvent) => {
      switch (event.type) {
        case 'MessageReceived':
          if (event.message) {
            setState(prev => {
              // Only add if it's for the current conversation
              if (prev.currentConversation && 
                  event.message!.message.conversation_id === prev.currentConversation.conversation.id) {
                return {
                  ...prev,
                  messages: [event.message!, ...prev.messages],
                };
              }
              return prev;
            });
          }
          break;

        case 'MessageStream':
          if (event.event) {
            const { message_id, chunk, is_complete } = event.event;
            setState(prev => {
              const newStreamingMessages = new Map(prev.streamingMessages);
              
              if (is_complete) {
                newStreamingMessages.delete(message_id);
              } else {
                const existing = newStreamingMessages.get(message_id) || '';
                newStreamingMessages.set(message_id, existing + chunk);
              }
              
              return {
                ...prev,
                streamingMessages: newStreamingMessages,
              };
            });
          }
          break;

        case 'ConversationUpdated':
          if (event.conversation) {
            setState(prev => {
              const updatedConversations = prev.conversations.map(conv =>
                conv.conversation.id === event.conversation!.conversation.id
                  ? event.conversation!
                  : conv
              );
              
              // Add if new
              if (!prev.conversations.find(c => c.conversation.id === event.conversation!.conversation.id)) {
                updatedConversations.push(event.conversation!);
              }
              
              return {
                ...prev,
                conversations: updatedConversations,
              };
            });
          }
          break;
      }
    };

    chatAPI.onEvent('chat-hook', handleChatEvent);
    return () => chatAPI.offEvent('chat-hook');
  }, []);

  // Load initial data
  useEffect(() => {
    if (userId) {
      loadConversations();
    }
  }, [userId, loadConversations]);

  return {
    ...state,
    pagination,
    actions: {
      loadConversations,
      loadMessages,
      sendMessage,
      selectConversation,
      loadMoreMessages: () => loadMessages(state.currentConversation!.conversation.id, pagination.page + 1),
    },
  };
};
```

### Unified Chat Component
```typescript
// src/components/Chat/ChatInterface.tsx
import React, { useState, useRef, useEffect } from 'react';
import { useChat } from '../../hooks/useChat';
import { ConversationWithParticipants, MessageWithSender, ConversationType, ParticipantType } from '../../types/api';
import { chatAPI } from '../../api/client';

interface ChatInterfaceProps {
  userId: string;
  onStartAgentChat?: (agentId: string) => void;
}

export const ChatInterface: React.FC<ChatInterfaceProps> = ({ userId, onStartAgentChat }) => {
  const {
    conversations,
    currentConversation,
    messages,
    streamingMessages,
    loading,
    error,
    pagination,
    actions,
  } = useChat(userId);

  const [messageInput, setMessageInput] = useState('');
  const [showAgentPanel, setShowAgentPanel] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages, streamingMessages]);

  const handleSendMessage = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!messageInput.trim()) return;

    await actions.sendMessage(messageInput);
    setMessageInput('');
  };

  const handleStartAgentChat = async (agentId: string) => {
    try {
      const conversation = await chatAPI.startAgentChat(agentId, userId);
      actions.selectConversation(conversation);
      setShowAgentPanel(false);
    } catch (error) {
      console.error('Failed to start agent chat:', error);
    }
  };

  const getConversationTitle = (conv: ConversationWithParticipants) => {
    if (conv.conversation.title) {
      return conv.conversation.title;
    }
    
    if (conv.conversation.conversation_type === ConversationType.AgentChat) {
      const agent = conv.participants.find(p => p.participant_type === ParticipantType.Agent);
      return agent ? `Chat with ${agent.display_name}` : 'Agent Chat';
    }
    
    const otherParticipants = conv.participants.filter(p => p.id !== userId);
    return otherParticipants.map(p => p.display_name).join(', ');
  };

  const isAgentConversation = (conv: ConversationWithParticipants) => {
    return conv.conversation.conversation_type === ConversationType.AgentChat;
  };

  return (
    <div className="flex h-screen bg-gray-100">
      {/* Sidebar */}
      <div className="w-1/3 bg-white border-r border-gray-200 flex flex-col">
        <div className="p-4 border-b border-gray-200">
          <div className="flex justify-between items-center mb-4">
            <h2 className="text-xl font-semibold">Conversations</h2>
            <button
              onClick={() => setShowAgentPanel(true)}
              className="bg-blue-500 hover:bg-blue-600 text-white px-3 py-1 rounded text-sm"
            >
              New Agent Chat
            </button>
          </div>
          <div className="relative">
            <input
              type="text"
              placeholder="Search conversations..."
              className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <svg className="absolute left-3 top-2.5 h-5 w-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </div>
        </div>

        <div className="flex-1 overflow-y-auto">
          {loading && conversations.length === 0 && (
            <div className="p-4 text-center text-gray-500">Loading conversations...</div>
          )}
          
          {conversations.map((conv) => (
            <div
              key={conv.conversation.id}
              onClick={() => actions.selectConversation(conv)}
              className={`p-4 border-b border-gray-100 cursor-pointer hover:bg-gray-50 ${
                currentConversation?.conversation.id === conv.conversation.id ? 'bg-blue-50 border-blue-200' : ''
              }`}
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-3">
                  <div className="w-10 h-10 bg-gray-300 rounded-full flex items-center justify-center">
                    {isAgentConversation(conv) ? (
                      <svg className="w-5 h-5 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                      </svg>
                    ) : (
                      <span className="text-sm font-medium text-gray-600">
                        {getConversationTitle(conv)[0]}
                      </span>
                    )}
                  </div>
                  <div className="flex-1">
                    <div className="flex items-center justify-between">
                      <h3 className="font-medium text-gray-900 truncate">
                        {getConversationTitle(conv)}
                      </h3>
                      {conv.unread_count > 0 && (
                        <span className="bg-blue-500 text-white text-xs rounded-full px-2 py-1">
                          {conv.unread_count}
                        </span>
                      )}
                    </div>
                    <p className="text-sm text-gray-500 truncate">
                      {conv.conversation.last_message_at
                        ? new Date(conv.conversation.last_message_at).toLocaleString()
                        : 'No messages yet'}
                    </p>
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Main Chat Area */}
      <div className="flex-1 flex flex-col">
        {currentConversation ? (
          <>
            {/* Chat Header */}
            <div className="bg-white border-b border-gray-200 px-6 py-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-3">
                  <div className="w-8 h-8 bg-gray-300 rounded-full flex items-center justify-center">
                    {isAgentConversation(currentConversation) ? (
                      <svg className="w-4 h-4 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                      </svg>
                    ) : (
                      <span className="text-sm font-medium text-gray-600">
                        {getConversationTitle(currentConversation)[0]}
                      </span>
                    )}
                  </div>
                  <div>
                    <h3 className="font-medium text-gray-900">
                      {getConversationTitle(currentConversation)}
                    </h3>
                    <p className="text-sm text-gray-500">
                      {currentConversation.participants.length} participant{currentConversation.participants.length !== 1 ? 's' : ''}
                    </p>
                  </div>
                </div>
                <div className="flex items-center space-x-2">
                  {isAgentConversation(currentConversation) && (
                    <span className="bg-green-100 text-green-800 text-xs px-2 py-1 rounded-full">
                      Agent
                    </span>
                  )}
                </div>
              </div>
            </div>

            {/* Messages */}
            <div className="flex-1 overflow-y-auto p-6 space-y-4">
              {pagination.hasNext && (
                <div className="text-center">
                  <button
                    onClick={() => actions.loadMoreMessages()}
                    className="text-blue-500 hover:text-blue-600 text-sm"
                  >
                    Load older messages
                  </button>
                </div>
              )}

              {messages.map((msgWithSender) => (
                <MessageComponent
                  key={msgWithSender.message.id}
                  message={msgWithSender}
                  currentUserId={userId}
                  streamingContent={streamingMessages.get(msgWithSender.message.id)}
                />
              ))}

              {/* Show streaming messages that don't have a full message yet */}
              {Array.from(streamingMessages.entries()).map(([messageId, content]) => (
                <div key={`streaming-${messageId}`} className="flex justify-start">
                  <div className="bg-gray-100 rounded-lg px-4 py-2 max-w-xs lg:max-w-md">
                    <p className="text-sm text-gray-700">{content}</p>
                    <div className="flex items-center mt-1">
                      <div className="animate-pulse w-2 h-2 bg-blue-500 rounded-full mr-1"></div>
                      <span className="text-xs text-gray-500">Typing...</span>
                    </div>
                  </div>
                </div>
              ))}

              <div ref={messagesEndRef} />
            </div>

            {/* Message Input */}
            <div className="bg-white border-t border-gray-200 px-6 py-4">
              <form onSubmit={handleSendMessage} className="flex space-x-4">
                <input
                  type="text"
                  value={messageInput}
                  onChange={(e) => setMessageInput(e.target.value)}
                  placeholder="Type a message..."
                  className="flex-1 border border-gray-300 rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
                <button
                  type="submit"
                  disabled={!messageInput.trim()}
                  className="bg-blue-500 hover:bg-blue-600 disabled:bg-gray-300 text-white px-6 py-2 rounded-lg"
                >
                  Send
                </button>
              </form>
            </div>
          </>
        ) : (
          <div className="flex-1 flex items-center justify-center">
            <div className="text-center">
              <svg className="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
              </svg>
              <h3 className="mt-2 text-sm font-medium text-gray-900">No conversation selected</h3>
              <p className="mt-1 text-sm text-gray-500">Choose a conversation to start chatting</p>
            </div>
          </div>
        )}
      </div>

      {/* Agent Panel Modal */}
      {showAgentPanel && (
        <AgentSelectionModal
          onClose={() => setShowAgentPanel(false)}
          onSelectAgent={handleStartAgentChat}
        />
      )}
    </div>
  );
};

interface MessageComponentProps {
  message: MessageWithSender;
  currentUserId: string;
  streamingContent?: string;
}

const MessageComponent: React.FC<MessageComponentProps> = ({ message, currentUserId, streamingContent }) => {
  const isOwn = message.sender.id === currentUserId;
  const isAgent = message.sender.participant_type === ParticipantType.Agent;

  return (
    <div className={`flex ${isOwn ? 'justify-end' : 'justify-start'}`}>
      <div className={`max-w-xs lg:max-w-md ${isOwn ? 'order-2' : 'order-1'}`}>
        <div className={`flex items-end space-x-2 ${isOwn ? 'flex-row-reverse space-x-reverse' : ''}`}>
          <div className="w-8 h-8 bg-gray-300 rounded-full flex items-center justify-center flex-shrink-0">
            {isAgent ? (
              <svg className="w-4 h-4 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
              </svg>
            ) : (
              <span className="text-xs font-medium text-gray-600">
                {message.sender.display_name[0]}
              </span>
            )}
          </div>
          <div className={`rounded-lg px-4 py-2 ${
            isOwn 
              ? 'bg-blue-500 text-white' 
              : isAgent 
                ? 'bg-green-100 text-green-800'
                : 'bg-gray-100 text-gray-900'
          }`}>
            {!isOwn && (
              <p className="text-xs font-medium mb-1 opacity-75">
                {message.sender.display_name}
              </p>
            )}
            <p className="text-sm">
              {streamingContent || message.message.content}
            </p>
            <p className={`text-xs mt-1 opacity-75`}>
              {new Date(message.message.sent_at).toLocaleTimeString()}
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};

interface AgentSelectionModalProps {
  onClose: () => void;
  onSelectAgent: (agentId: string) => void;
}

const AgentSelectionModal: React.FC<AgentSelectionModalProps> = ({ onClose, onSelectAgent }) => {
  // Mock agents - in real app, these would come from an API
  const agents = [
    { id: 'agent-1', name: 'General Assistant', description: 'General purpose AI assistant' },
    { id: 'agent-2', name: 'Code Helper', description: 'Specialized in coding and development' },
    { id: 'agent-3', name: 'Writing Assistant', description: 'Helps with writing and editing' },
  ];

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg p-6 w-96">
        <div className="flex justify-between items-center mb-4">
          <h3 className="text-lg font-semibold">Select an Agent</h3>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        
        <div className="space-y-3">
          {agents.map((agent) => (
            <button
              key={agent.id}
              onClick={() => onSelectAgent(agent.id)}
              className="w-full p-3 text-left border border-gray-200 rounded-lg hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <div className="font-medium text-gray-900">{agent.name}</div>
              <div className="text-sm text-gray-500">{agent.description}</div>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
};
```

### App Integration
```typescript
// src/App.tsx
import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { ChatInterface } from './components/Chat/ChatInterface';

const App: React.FC = () => {
  // In a real app, this would come from authentication
  const currentUserId = 'user-123';

  return (
    <Router>
      <div className="App">
        <Routes>
          <Route 
            path="/" 
            element={
              <ChatInterface 
                userId={currentUserId}
                onStartAgentChat={(agentId) => console.log('Starting chat with agent:', agentId)}
              />
            } 
          />
          <Route path="/chat/:conversationId" element={<ChatInterface userId={currentUserId} />} />
        </Routes>
      </div>
    </Router>
  );
};

export default App;
```

## **Key Features**

1. **Unified Data Model**: Both user-to-user and user-to-agent chats use the same participant and conversation model
2. **Real-time Updates**: WebSocket-like events through Tauri's event system
3. **Streaming Support**: Built-in streaming for agent responses
4. **Type Safety**: Full TypeScript support with shared types
5. **Easy Schema Changes**: Migration system for database updates
6. **Reusable Components**: Same chat interface for both use cases
7. **Pagination**: Built-in pagination for conversations and messages
8. **Error Handling**: Comprehensive error handling throughout
9. **State Management**: Custom React hooks for state management
10. **Responsive Design**: Mobile-friendly interface with Tailwind CSS

This framework provides a solid foundation that you can easily extend as your application grows!


---

## From Gemini

Of course. This is an excellent, real-world scenario that combines several modern technologies. I will outline a complete framework that is type-safe, easy to manage, and supports streaming for both your use cases.

The key to this architecture is `tauri-specta`, a phenomenal tool that provides end-to-end type safety between your Rust backend and TypeScript frontend, including the commands themselves.

### Core Architecture Principles

1.  **Single Source of Truth for Types:** Rust structs in your backend are the definitive source. We'll automatically generate TypeScript types from them.
2.  **Explicit API Layer:** Tauri commands define your API. `tauri-specta` will generate a typed client for the frontend to call these commands.
3.  **Event-Driven Streaming:** We will use Tauri's built-in event system for streaming. The backend will emit events, and the frontend will listen for them. This is perfect for real-time messages (libp2p) and agent responses.
4.  **Centralized Frontend State:** We'll use `zustand` for simple, powerful global state management in React.
5.  **Database Managed by Migrations:** We'll use `sqlx-cli` to manage schema and seed data, ensuring that changes are easy and repeatable.

---

### 1. Backend Setup (Tauri, SQLx, `tauri-specta`)

#### a. Project Structure (`src-tauri/`)

```
src-tauri/
├── Cargo.toml
├── build.rs             # For tauri-specta
├── migrations/
│   └── 202311..._init.sql
├── tauri.conf.json
└── src/
    ├── api.rs           # All Tauri commands
    ├── db.rs            # Database setup & migrations
    ├── error.rs         # Custom error type
    ├── main.rs          # Main app setup
    ├── models.rs        # Core data structures (Rust structs)
    ├── p2p.rs           # libp2p service logic
    └── agent.rs         # Agent service logic
```

#### b. Dependencies (`src-tauri/Cargo.toml`)

```toml
[dependencies]
# Tauri & SQLx
tauri = { version = "2.0.0-beta", features = [] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "macros", "migrate"] }

# Type Generation & API Layer
specta = "2.0.0-rc.8"
tauri-specta = { version = "2.0.0-rc.5", features = ["javascript", "typescript"] }
# NOTE: Use specta::Type and ts_rs::TS for full compatibility
ts-rs = { version = "7.0", features = ["chrono-impl", "uuid-impl"] }

# Async & Utilities
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }

[build-dependencies]
tauri-specta = { version = "2.0.0-rc.5", features = ["javascript", "typescript"] }
```

#### c. Type Generation (`build.rs` and `models.rs`)

**`src-tauri/src/models.rs`** - This is your single source of truth.

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type, TS, sqlx::FromRow)]
#[ts(export, export_to = "../src/bindings/")] // Exports to frontend
pub enum ParticipantType {
    User,
    Agent,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, TS, sqlx::FromRow)]
#[ts(export, export_to = "../src/bindings/")]
pub struct Participant {
    pub id: String, // For a User, this is their PeerID. For an Agent, a unique name.
    pub participant_type: ParticipantType,
    pub alias: String, // A user-friendly name
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, TS, sqlx::FromRow)]
#[ts(export, export_to = "../src/bindings/")]
pub struct Conversation {
    #[ts(type = "string")] // Represent Uuid as a string in TS
    pub id: Uuid,
    pub participant_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, TS, sqlx::FromRow)]
#[ts(export, export_to = "../src/bindings/")]
pub struct Message {
    #[ts(type = "string")]
    pub id: Uuid,
    #[ts(type = "string")]
    pub conversation_id: Uuid,
    pub author_id: String, // The Participant ID
    pub content: String,
    pub sent_at: DateTime<Utc>,
}
```

**`src-tauri/build.rs`** - This script runs before compilation.

```rust
fn main() {
    // This will generate a file at src/bindings.ts
    // containing your types and a typed client.
    tauri_specta::ts::export(
        specta::collect_types![
            // List all your command functions here
            crate::api::list_conversations,
            crate::api::create_conversation,
            crate::api::get_messages,
            crate::api::send_p2p_message,
            crate::api::stream_agent_response
        ]
        .unwrap(),
        "../src/bindings.ts",
    )
    .unwrap();
}
```

#### d. API Commands & Streaming (`api.rs`)

```rust
use tauri::{command, AppHandle, Manager, State};
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use crate::{models::*, AppState, Result}; // Your custom Result type

#[command]
#[specta::specta] // This is required for tauri-specta
pub async fn list_conversations(state: State<'_, AppState>) -> Result<Vec<Conversation>> {
    // ... query database using state.db ...
    Ok(vec![]) // Placeholder
}

#[command]
#[specta::specta]
pub async fn create_conversation(state: State<'_, AppState>, participant_id: String) -> Result<Conversation> {
    // ... insert into database and return the new conversation ...
    unimplemented!()
}

#[command]
#[specta::specta]
pub async fn get_messages(state: State<'_, AppState>, conversation_id: Uuid) -> Result<Vec<Message>> {
    // ... get messages from DB for a given conversation ...
    Ok(vec![])
}

#[command]
#[specta::specta]
pub async fn send_p2p_message(
    state: State<'_, AppState>,
    conversation_id: Uuid,
    content: String,
) -> Result<Message> {
    // 1. Create and save the message to your local DB first
    // 2. Send the message via your libp2p service
    // 3. Return the saved message so the UI can show it immediately
    unimplemented!()
}

// THE STREAMING COMMAND
#[command]
#[specta::specta]
pub async fn stream_agent_response(
    app_handle: AppHandle,
    conversation_id: Uuid,
    prompt: String,
) -> Result<()> {
    // This command starts the process but returns immediately.
    // The actual streaming happens in a spawned task.
    tokio::spawn(async move {
        let conversation_str = conversation_id.to_string();
        let chunks = vec!["Hello! ", "I am ", "a streaming ", "agent. ", "How can ", "I help ", "you?"];
        
        for (i, chunk) in chunks.iter().enumerate() {
            // Simulate work
            sleep(Duration::from_millis(150)).await;

            // Emit an event to the frontend
            app_handle.emit_to("main", "agent_chunk", AgentChunk {
                conversation_id: conversation_str.clone(),
                content: chunk.to_string(),
                is_last: i == chunks.len() - 1,
            }).unwrap();
        }
    });

    Ok(())
}

// The payload for our streaming event
#[derive(Debug, Clone, Serialize, Type, TS)]
#[ts(export, export_to = "../src/bindings/")]
pub struct AgentChunk {
    pub conversation_id: String,
    pub content: String,
    pub is_last: bool,
}
```

---

### 2. Frontend Setup (React, Zustand, `tauri-specta`)

#### a. Dependencies (`package.json`)

```json
{
  "scripts": {
    "dev": "vite",
    "build": "pnpm build:types && vite build",
    "build:types": "cargo build -p app" // This triggers the `build.rs` script
  },
  "dependencies": {
    "@tauri-apps/api": "2.0.0-beta.8",
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "react-router-dom": "^6.21.1",
    "zustand": "^4.4.7"
  },
  "devDependencies": {
    // ... vite, typescript, tailwindcss ...
  }
}
```

#### b. Generated Bindings (`src/bindings.ts`)

After running `pnpm build:types`, this file will be created automatically. It's the magic glue.

```typescript
// This file was generated by [tauri-specta](https://github.com/oscartbeaumont/tauri-specta). Do not edit this file manually.

// And here we export the types that were generated in your other files.
export * from "./bindings/AgentChunk"
export * from "./bindings/Conversation"
// ... etc

import { invoke as TAURI_INVOKE } from "@tauri-apps/api/core";

// This is a typed wrapper around Tauri's invoke function
export const commands = {
    async listConversations(): Promise<Conversation[]> {
        return await TAURI_INVOKE("list_conversations");
    },
    async createConversation(participantId: string): Promise<Conversation> {
        return await TAURI_INVOKE("create_conversation", { participantId });
    },
    // ... and so on for all your commands
    async streamAgentResponse(conversationId: string, prompt: string): Promise<null> {
        return await TAURI_INVOKE("stream_agent_response", { conversationId, prompt });
    }
}
```

#### c. State Management (`src/store.ts`)

```typescript
import { create } from 'zustand';
import { Conversation, Message } from './bindings'; // Import generated types

interface ChatState {
  conversations: Conversation[];
  messages: Record<string, Message[]>; // conversationId -> messages[]
  agentStreamingResponse: Record<string, string>; // conversationId -> streaming text
  fetchConversations: () => Promise<void>;
  addMessage: (message: Message) => void;
  appendAgentChunk: (conversationId: string, chunk: string) => void;
  clearAgentStream: (conversationId: string) => void;
}

// This is where we will use our typed API client
import { commands } from './bindings';

export const useChatStore = create<ChatState>((set, get) => ({
  conversations: [],
  messages: {},
  agentStreamingResponse: {},
  fetchConversations: async () => {
    const conversations = await commands.listConversations();
    set({ conversations });
  },
  addMessage: (message) => {
    set((state) => ({
      messages: {
        ...state.messages,
        [message.conversation_id]: [...(state.messages[message.conversation_id] || []), message],
      },
    }));
  },
  appendAgentChunk: (conversationId, chunk) => {
      set(state => ({
          agentStreamingResponse: {
              ...state.agentStreamingResponse,
              [conversationId]: (state.agentStreamingResponse[conversationId] || "") + chunk
          }
      }))
  },
  clearAgentStream: (conversationId: string) => {
    const finalMessageContent = get().agentStreamingResponse[conversationId];
    // Here you would create a final message and add it to the main message list
    // ... get().addMessage(...) ...
    set(state => {
        const newStreams = {...state.agentStreamingResponse};
        delete newStreams[conversationId];
        return { agentStreamingResponse: newStreams };
    });
  }
}));
```

#### d. Event Handling Hook (`src/hooks/useChatEvents.ts`)

This hook cleanly encapsulates all backend event listeners.

```typescript
import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useChatStore } from '../store';
import { Message, AgentChunk } from '../bindings';

export const useChatEvents = () => {
  const { addMessage, appendAgentChunk, clearAgentStream } = useChatStore();

  useEffect(() => {
    // Listener for new p2p messages
    const unlistenP2p = listen<Message>('new_p2p_message', (event) => {
      console.log('Received new p2p message:', event.payload);
      addMessage(event.payload);
    });

    // Listener for agent streaming chunks
    const unlistenAgent = listen<AgentChunk>('agent_chunk', (event) => {
        const { conversation_id, content, is_last } = event.payload;
        appendAgentChunk(conversation_id, content);
        if (is_last) {
            clearAgentStream(conversation_id);
        }
    });

    return () => {
      // Cleanup on component unmount
      unlistenP2p.then((fn) => fn());
      unlistenAgent.then((fn) => fn());
    };
  }, [addMessage, appendAgentChunk, clearAgentStream]);
};
```

#### e. UI Component (`src/components/ChatView.tsx`)

This component brings everything together for a single chat session.

```tsx
import { useParams } from 'react-router-dom';
import { useChatStore } from '../store';
import { useChatEvents } from '../hooks/useChatEvents';
import { commands } from '../bindings'; // Our typed commands
import { FormEvent } from 'react';

export function ChatView() {
  // 1. Get conversation ID from URL
  const { conversationId } = useParams<{ conversationId: string }>();
  
  // 2. Setup event listeners
  useChatEvents();

  // 3. Get data and actions from our global store
  const messages = useChatStore((s) => s.messages[conversationId!] || []);
  const streamingResponse = useChatStore((s) => s.agentStreamingResponse[conversationId!]);

  // A simple way to determine if we are talking to an agent
  const isAgentChat = conversationId === 'agent-123';

  const handleSubmit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const content = formData.get('message') as string;
    if (!content || !conversationId) return;
    
    e.currentTarget.reset();

    if (isAgentChat) {
        // Use the agent streaming command
        await commands.streamAgentResponse(conversationId, content);
    } else {
        // Use the p2p message command
        const savedMessage = await commands.sendP2pMessage(conversationId, content);
        // We can optimistically add the message to the store here if needed
    }
  };

  return (
    <div className="flex flex-col h-screen p-4">
      <div className="flex-1 overflow-y-auto">
        {messages.map((msg) => (
          <div key={msg.id}>{msg.content}</div>
        ))}
        {/* Render the in-progress streaming response */}
        {streamingResponse && <div className="text-gray-500">{streamingResponse}</div>}
      </div>
      <form onSubmit={handleSubmit} className="mt-4 flex">
        <input name="message" className="flex-1 p-2 border rounded-l-md" />
        <button type="submit" className="px-4 py-2 bg-blue-500 text-white rounded-r-md">Send</button>
      </form>
    </div>
  );
}
```

### Development Workflow & Schema Changes

This setup makes development a breeze:

1.  **Need to change a model?**
    *   Edit the struct in `src-tauri/src/models.rs`.
    *   Run `pnpm build:types`.
    *   Your TypeScript code will now show compile-time errors anywhere the old type is used. Fix them.

2.  **Need to add a new API endpoint?**
    *   Add a new `#[command]` function in `src-tauri/src/api.rs`.
    *   Add its name to the `specta::collect_types!` macro in `build.rs`.
    *   Run `pnpm build:types`.
    *   The new function is now available and fully typed on `commands` in your frontend.

3.  **Need to change the database?**
    *   `sqlx migrate add my_new_change`
    *   Edit the new SQL migration file.
    *   The next time the Tauri app starts, `sqlx::migrate!` will apply the changes automatically.

This framework provides a robust, scalable, and highly developer-friendly foundation for your application, fully supporting type-safety and real-time streaming for both your P2P and Agent chat use cases.


---

## From GPT

Here's a structured, maintainable, and type-safe example architecture designed specifically for your described use cases. It integrates smoothly between a Tauri v2 backend (Rust + SQLx + SQLite), and a modern frontend stack (Vite, React, React Router, TypeScript, Tailwind). This approach includes schema-driven API management, real-time streaming, and flexible state/types management, facilitating easy schema adaptation during development.

---

## 🛠️ Tech Stack Overview:

### Backend (Tauri v2 + Rust):
- **Database**: SQLite (managed via SQLx)
- **API Layer**: Tauri commands as backend API endpoints
- **Schema Management**: `sqlx-cli` migrations
- **Real-time streaming**: Libp2p for peer-to-peer messages, Tauri event streams (`emit`/`listen`) for frontend streaming
- **Type Safety**: Rust structs exposed via Tauri APIs; auto-generate TypeScript types via `tauri-bindgen`

### Frontend (Vite + React):
- **Bundler**: Vite (fast HMR)
- **UI Framework**: React + TypeScript + TailwindCSS
- **Routing**: React Router
- **State Management**: Zustand (lightweight), React Query or SWR (data-fetching)
- **Real-time communication**: Tauri events for streaming updates

---

## 📂 Recommended Project Structure:

```
my-chat-app/
├── src-tauri/
│   ├── Cargo.toml
│   ├── migrations/ (sqlx-cli migration files)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── db.rs
│   │   ├── api.rs
│   │   ├── models.rs
│   │   ├── state.rs
│   │   └── libp2p.rs
│   └── tauri.conf.json
│
└── frontend/ (vite-react frontend)
    ├── src/
    │   ├── components/
    │   ├── pages/
    │   ├── api/ (Tauri API client)
    │   ├── types.ts (auto-generated types)
    │   ├── state/ (zustand stores)
    │   └── main.tsx
    ├── tailwind.config.js
    └── vite.config.ts
```

---

## 🧑‍💻 Backend Implementation:

### ✅ Rust data models (`src-tauri/src/models.rs`):

```rust
use serde::{Serialize, Deserialize};

// Participant type enum
#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum ParticipantType {
    User,
    Agent,
}

// Participant model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Participant {
    pub id: String,
    pub participant_type: ParticipantType,
}

// Conversation model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Conversation {
    pub id: i64,
    pub created_at: chrono::NaiveDateTime,
}

// Message model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: i64,
    pub conversation_id: i64,
    pub sender_id: String,
    pub content: String,
    pub timestamp: chrono::NaiveDateTime,
}
```

---

### ✅ API layer (`src-tauri/src/api.rs`):

```rust
use crate::{db::DbState, models::*};
use tauri::{command, State};
use sqlx::SqlitePool;

#[command]
pub async fn create_conversation(
    participants: Vec<Participant>, 
    state: State<'_, DbState>
) -> Result<Conversation, String> {
    let pool = &state.pool;

    let conv = sqlx::query_as!(
        Conversation,
        r#"
        INSERT INTO conversations DEFAULT VALUES RETURNING id, created_at;
        "#
    )
    .fetch_one(pool)
    .await.map_err(|e| e.to_string())?;

    // insert participants logic here...

    Ok(conv)
}

#[command]
pub async fn send_message(
    conversation_id: i64,
    sender_id: String,
    content: String,
    state: State<'_, DbState>,
    app_handle: tauri::AppHandle,
) -> Result<Message, String> {
    let pool = &state.pool;

    let msg = sqlx::query_as!(
        Message,
        r#"
        INSERT INTO messages (conversation_id, sender_id, content) 
        VALUES (?, ?, ?)
        RETURNING id, conversation_id, sender_id, content, timestamp;
        "#,
        conversation_id,
        sender_id,
        content
    )
    .fetch_one(pool)
    .await.map_err(|e| e.to_string())?;

    // Emit event to frontend listeners for streaming updates
    app_handle.emit_all("new-message", &msg).unwrap();

    Ok(msg)
}
```

---

### ✅ Libp2p integration (`src-tauri/src/libp2p.rs`):

```rust
// simplified example of libp2p setup for peer-to-peer messages
use libp2p::{identity, PeerId};

pub fn generate_peer_id() -> PeerId {
  let keypair = identity::Keypair::generate_ed25519();
  keypair.public().to_peer_id()
}

// store peer ids and handle direct message logic here...
```

---

## 🎨 Frontend Implementation:

### ✅ Auto-generated API & Types (`frontend/src/api/tauri.ts`):

Use `tauri-bindgen` or manually define methods:

```typescript
import { invoke } from "@tauri-apps/api";

export const createConversation = async (participants) => 
  invoke<Conversation>("create_conversation", { participants });

export const sendMessage = async ({conversationId, senderId, content}) =>
  invoke<Message>("send_message", { conversationId, senderId, content });
```

### ✅ Zustand Store (`frontend/src/state/chatStore.ts`):

```typescript
import { create } from 'zustand';

interface ChatState {
  conversationId?: number;
  messages: Message[];
  setConversationId: (id: number) => void;
  addMessage: (msg: Message) => void;
}

export const useChatStore = create<ChatState>((set) => ({
  conversationId: undefined,
  messages: [],
  setConversationId: (conversationId) => set({ conversationId }),
  addMessage: (msg) => set((state) => ({ messages: [...state.messages, msg] })),
}));
```

### ✅ React Streaming Listener (`frontend/src/hooks/useMessageStream.ts`):

```typescript
import { listen } from '@tauri-apps/api/event';
import { useEffect } from 'react';
import { useChatStore } from '../state/chatStore';

export function useMessageStream() {
  const addMessage = useChatStore(state => state.addMessage);

  useEffect(() => {
    const unlisten = listen<Message>('new-message', event => {
      addMessage(event.payload);
    });

    return () => { unlisten.then(f => f()); };
  }, []);
}
```

### ✅ UI Component Example (`frontend/src/components/Chat.tsx`):

```tsx
export const Chat = () => {
  const { messages } = useChatStore();
  useMessageStream();

  return (
    <div className="chat-container">
      {messages.map(msg => (
        <div key={msg.id}>{msg.content}</div>
      ))}
    </div>
  );
};
```

---

## 🚩 Schema Changes & Adaptation Workflow:

- Update SQL schema via `sqlx-cli` migrations.
- Adjust Rust structs accordingly.
- Frontend types update automatically via Tauri's type exports or manual sync.
- Vite's fast reload/HMR workflow ensures quick iteration on frontend components.

---

## 🎯 Summary of Key Benefits:

- **Easy Schema Adaptation**: SQLx migrations streamline DB changes.
- **Type-safety & State Management**: Strong Rust + TS typing; Zustand provides simple state management.
- **Unified Chat Framework**: Same UI components and backend models used for:
  - User-to-user direct messaging via libp2p.
  - User-to-agent streaming sessions via Tauri event streams.

This architecture provides a robust yet flexible foundation suitable for rapid development of your described scenarios.