pub mod tickets;

use std::{collections::HashMap, sync::Arc};
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Query},
    response::Response,
    Extension, http::StatusCode,
};
use tokio::sync::{mpsc, RwLock};
use tokio::time::{timeout, Duration};
use futures_util::{StreamExt, SinkExt};
use serde::Deserialize;
use sqlx::PgPool;
use chrono::Utc;
use tickets::WsTickets;
use crate::models::message::{ClientEvent, ServerEvent};

const PING_INTERVAL: Duration = Duration::from_secs(30);
const PONG_TIMEOUT: Duration  = Duration::from_secs(10);

pub type ConnectedUsers = Arc<RwLock<HashMap<i64, mpsc::UnboundedSender<Message>>>>;

#[derive(Deserialize)]
pub struct WsQuery {
    ticket: String,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    Extension(tickets): Extension<WsTickets>,
    Extension(users): Extension<ConnectedUsers>,
    Extension(pool): Extension<PgPool>,
) -> Result<Response, StatusCode> {
    let ticket = tickets::redeem_ticket(&query.ticket, &tickets).await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, ticket.user_id, ticket.username, users, pool)))
}

async fn handle_socket(socket: WebSocket, user_id: i64, username: String, users: ConnectedUsers, pool: PgPool) {
    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    users.write().await.insert(user_id, tx);
    tracing::info!("{} (id={}) connected", username, user_id);

    // send Hello immediately so the frontend can confirm identity + sync clock on reconnect
    let hello = serde_json::to_string(&ServerEvent::Hello {
        user_id,
        server_time: Utc::now(),
    }).unwrap();
    let _ = sink.send(Message::Text(hello.into())).await;

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sink.send(msg).await.is_err() {
                break;
            }
        }
    });

    // ping task: fires every PING_INTERVAL, expects a Pong back within PONG_TIMEOUT
    // if no pong arrives the socket is considered dead — we drop the tx which closes send_task
    let (pong_tx, mut pong_rx) = mpsc::unbounded_channel::<()>();
    let users_ping = users.clone();
    let ping_task = tokio::spawn(async move {
        loop {
            tokio::time::sleep(PING_INTERVAL).await;
            send_event(&users_ping, user_id, &ServerEvent::Ping).await;
            if timeout(PONG_TIMEOUT, pong_rx.recv()).await.is_err() {
                tracing::warn!("ws: user={} pong timeout, closing", user_id);
                break;
            }
        }
    });

    while let Some(Ok(Message::Text(text))) = stream.next().await {
        let event = match serde_json::from_str::<ClientEvent>(&text) {
            Ok(e) => e,
            Err(_) => {
                tracing::warn!("ws: user={} sent invalid event", user_id);
                send_event(&users, user_id, &ServerEvent::Error { reason: "invalid event" }).await;
                continue;
            }
        };

        match event {
            ClientEvent::SendMessage { recipient_id, content } => {
                handle_send_message(&pool, &users, user_id, recipient_id, content).await;
            }
            ClientEvent::MarkRead { conversation_id } => {
                handle_mark_read(&pool, &users, user_id, conversation_id).await;
            }
            ClientEvent::Pong => {
                let _ = pong_tx.send(());
            }
        }
    }

    ping_task.abort();
    send_task.abort();
    users.write().await.remove(&user_id);
    tracing::info!("{} (id={}) disconnected", username, user_id);
}

async fn handle_send_message(
    pool: &PgPool,
    users: &ConnectedUsers,
    sender_id: i64,
    recipient_id: i64,
    content: String,
) {
    if content.is_empty() || content.len() > 2000 {
        tracing::warn!("ws: user={} sent message with invalid length ({})", sender_id, content.len());
        send_event(users, sender_id, &ServerEvent::Error { reason: "invalid message length" }).await;
        return;
    }

    // get or create conversation, always store user1_id < user2_id
    let (user1_id, user2_id) = if sender_id < recipient_id {
        (sender_id, recipient_id)
    } else {
        (recipient_id, sender_id)
    };

    let conversation_id: i64 = match sqlx::query_scalar(
        r#"
        INSERT INTO conversations (user1_id, user2_id)
        VALUES ($1, $2)
        ON CONFLICT (user1_id, user2_id) DO UPDATE SET id = conversations.id
        RETURNING id
        "#,
    )
    .bind(user1_id)
    .bind(user2_id)
    .fetch_one(pool)
    .await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("ws: failed to get/create conversation sender={} recipient={}: {}", sender_id, recipient_id, e);
            send_event(users, sender_id, &ServerEvent::Error { reason: "internal error" }).await;
            return;
        }
    };

    let message = match sqlx::query_as(
        r#"
        INSERT INTO messages (conversation_id, sender_id, content)
        VALUES ($1, $2, $3)
        RETURNING id, conversation_id, sender_id, content, created_at
        "#,
    )
    .bind(conversation_id)
    .bind(sender_id)
    .bind(&content)
    .fetch_one(pool)
    .await {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("ws: failed to insert message sender={} convo={}: {}", sender_id, conversation_id, e);
            send_event(users, sender_id, &ServerEvent::Error { reason: "internal error" }).await;
            return;
        }
    };

    let event = ServerEvent::NewMessage { message: &message };
    // echo to sender + deliver to recipient if online
    send_event(users, sender_id, &event).await;
    send_event(users, recipient_id, &event).await;
}

async fn handle_mark_read(
    pool: &PgPool,
    users: &ConnectedUsers,
    user_id: i64,
    conversation_id: i64,
) {
    // insert read receipts for all unread messages in this conversation
    let result = sqlx::query(
        r#"
        INSERT INTO read_receipts (message_id, user_id)
        SELECT m.id, $1
        FROM messages m
        WHERE m.conversation_id = $2
          AND m.sender_id != $1
          AND NOT EXISTS (
              SELECT 1 FROM read_receipts rr
              WHERE rr.message_id = m.id AND rr.user_id = $1
          )
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(conversation_id)
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::error!("ws: failed to mark read user={} convo={}: {}", user_id, conversation_id, e);
        return;
    }

    // find the other participant and notify them if online
    let other_user_id: Option<i64> = sqlx::query_scalar(
        "SELECT CASE WHEN user1_id = $1 THEN user2_id ELSE user1_id END FROM conversations WHERE id = $2"
    )
    .bind(user_id)
    .bind(conversation_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    if let Some(other_id) = other_user_id {
        send_event(users, other_id, &ServerEvent::MessageRead {
            conversation_id,
            reader_id: user_id,
        }).await;
    }
}

async fn send_event(users: &ConnectedUsers, user_id: i64, event: &ServerEvent<'_>) {
    let Ok(json) = serde_json::to_string(event) else { return };
    if let Some(tx) = users.read().await.get(&user_id) {
        let _ = tx.send(Message::Text(json.into()));
    }
}
