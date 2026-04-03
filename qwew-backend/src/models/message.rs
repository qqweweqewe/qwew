use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Message {
    pub id: i64,
    pub conversation_id: i64,
    pub sender_id: i64,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Conversation {
    pub id: i64,
    pub user1_id: i64,
    pub user2_id: i64,
    pub created_at: DateTime<Utc>,
    // joined from users
    pub other_username: String,
    // joined from messages
    pub last_message: Option<String>,
    pub last_message_at: Option<DateTime<Utc>>,
    pub unread_count: i64,
}

// incoming WS events from client
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientEvent {
    SendMessage {
        recipient_id: i64,
        content: String,
    },
    MarkRead {
        conversation_id: i64,
    },
    Pong,
}

// outgoing WS events to client
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent<'a> {
    // sent immediately on connect so the frontend can confirm identity + sync clock
    Hello {
        user_id: i64,
        server_time: DateTime<Utc>,
    },
    NewMessage {
        message: &'a Message,
    },
    MessageRead {
        conversation_id: i64,
        reader_id: i64,
    },
    // server-initiated keepalive — frontend must respond with { "type": "pong" }
    Ping,
    Error {
        reason: &'a str,
    },
}
